import { describe, expect, it } from "bun:test";
import {
  getAbsenceIcalUrlFromContact,
  getDayliteContactDisplayName,
  getPrimaryAddressFromContact,
  getPrimaryIcalUrlFromContact,
  isAssignment,
  isDayliteContactRecord,
  isDayliteProjectRecord,
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
      ).toBe("");
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

    it("extracts iCal urls from urls and extra_fields", () => {
      const fromUrls = {
        self: "/v1/contacts/1000",
        urls: [
          {
            label: "Einsatz iCal",
            url: "https://example.com/primary.ics",
          },
          {
            label: "Abwesenheit iCal",
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

      const fromExtraFields = {
        self: "/v1/contacts/4000",
        extra_fields:
          '{"primary_ical_url":{"value":"https://example.com/primary.ics"},"absence_ical_url":{"value":"https://example.com/absence.ics"}}',
      };

      expect(getPrimaryIcalUrlFromContact(fromExtraFields)).toBe(
        "https://example.com/primary.ics",
      );
      expect(getAbsenceIcalUrlFromContact(fromExtraFields)).toBe(
        "https://example.com/absence.ics",
      );
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
        source: "app",
        syncStatus: "synced",
      };

      expect(isAssignment(assignment)).toBe(true);
      expect(isAssignment({ ...assignment, source: "other" })).toBe(false);
    });
  });
});
