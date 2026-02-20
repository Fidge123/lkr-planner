import { describe, expect, it } from "bun:test";
import {
  getAbsenceIcalUrlFromContact,
  getDayliteContactDisplayName,
  getPrimaryAddressFromContact,
  getPrimaryIcalUrlFromContact,
  isAssignment,
  isDayliteContactRecord,
  isDayliteProjectRecord,
  upsertDayliteContactIcalUrls,
} from "./planning";

describe("planning domain mappers and guards", () => {
  describe("isDayliteProjectRecord", () => {
    it("accepts valid project records from daylite", () => {
      const raw: unknown = {
        self: "/v1/projects/7000",
        name: "Sell Sea Shells",
        status: "new_status",
      };

      expect(isDayliteProjectRecord(raw)).toBe(true);
    });

    it("rejects invalid records", () => {
      const raw: unknown = {
        self: 7000,
        name: "Sell Sea Shells",
      };

      expect(isDayliteProjectRecord(raw)).toBe(false);
    });
  });

  describe("isDayliteContactRecord", () => {
    it("accepts valid contact records from daylite", () => {
      const raw: unknown = {
        self: "/v1/contacts/1000",
        first_name: "Thomas",
        last_name: "Bartelmess",
      };

      expect(isDayliteContactRecord(raw)).toBe(true);
    });
  });

  describe("daylite contact helpers", () => {
    it("uses nickname first, then full_name for display", () => {
      expect(
        getDayliteContactDisplayName({
          self: "/v1/contacts/1000",
          nickname: "Tom",
          full_name: "Thomas Bartelmess",
        }),
      ).toBe("Tom");

      expect(
        getDayliteContactDisplayName({
          self: "/v1/contacts/2000",
          full_name: "Anna Schmidt",
        }),
      ).toBe("Anna Schmidt");

      expect(
        getDayliteContactDisplayName({
          self: "/v1/contacts/3000",
        }),
      ).toBe("Unbenannter Kontakt");
    });

    it("returns daylite address format without remapping", () => {
      const contact = {
        self: "/v1/contacts/1000",
        addresses: [
          {
            label: "Home",
            street: "Musterstraße 1",
            city: "Köln",
            postal_code: "50667",
            country: "Deutschland",
          },
        ],
      };

      expect(getPrimaryAddressFromContact(contact)).toEqual({
        label: "Home",
        street: "Musterstraße 1",
        city: "Köln",
        postal_code: "50667",
        country: "Deutschland",
      });
    });

    it("extracts iCal urls from urls only", () => {
      const fromUrls = {
        self: "/v1/contacts/1000",
        urls: [
          {
            label: "FR-Termine",
            url: "https://example.com/primary.ics",
          },
          {
            label: "FR-Fehlzeiten",
            url: "https://example.com/absence.ics",
          },
        ],
      };

      expect(getPrimaryIcalUrlFromContact(fromUrls)).toBe(
        "https://example.com/primary.ics",
      );
      expect(getAbsenceIcalUrlFromContact(fromUrls)).toBe(
        "https://example.com/absence.ics",
      );

      expect(
        getPrimaryIcalUrlFromContact({
          self: "/v1/contacts/4000",
        }),
      ).toBe("");
      expect(
        getAbsenceIcalUrlFromContact({
          self: "/v1/contacts/4000",
        }),
      ).toBe("");
    });

    it("maps both iCal urls back to daylite urls while preserving unrelated urls", () => {
      const mergedUrls = upsertDayliteContactIcalUrls(
        [
          {
            label: "Website",
            url: "https://example.com",
          },
          {
            label: "Abwesenheit iCal",
            url: "https://old.example.com/absence.ics",
          },
        ],
        "https://new.example.com/primary.ics",
        "https://new.example.com/absence.ics",
      );

      expect(mergedUrls).toEqual([
        {
          label: "Website",
          url: "https://example.com",
        },
        {
          label: "Einsatz iCal",
          url: "https://new.example.com/primary.ics",
        },
        {
          label: "Abwesenheit iCal",
          url: "https://new.example.com/absence.ics",
        },
      ]);
    });
  });

  describe("isAssignment", () => {
    it("validates assignment shape and enums", () => {
      const assignment = {
        id: "asg-1",
        employeeId: "emp-1",
        projectId: "proj-1",
        period: {
          startDate: "2026-01-26",
          endDate: "2026-01-26",
        },
      };

      expect(isAssignment(assignment)).toBe(true);
    });
  });
});
