import { describe, expect, it } from "bun:test";
import {
  isAssignment,
  isDayliteContactRecord,
  isDayliteProjectRecord,
  isSyncIssue,
  mapDayliteContactRecordToEmployee,
  mapDayliteProjectRecordToProject,
} from "./planning";

describe("planning domain mappers and guards", () => {
  describe("isDayliteProjectRecord", () => {
    it("accepts valid project records from daylite", () => {
      const raw: unknown = {
        self: "/v1/projects/7000",
        name: "Sell Sea Shells",
        status: "new",
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

  describe("mapDayliteProjectRecordToProject", () => {
    it("maps daylite records to project domain type", () => {
      const project = mapDayliteProjectRecordToProject({
        self: "/v1/projects/7000",
        name: "Sell Sea Shells",
        status: "new",
      });

      expect(project).toEqual({
        id: "7000",
        dayliteReference: "/v1/projects/7000",
        name: "Sell Sea Shells",
        status: "new",
      });
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

    it("rejects invalid records", () => {
      const raw: unknown = {
        first_name: "Thomas",
      };

      expect(isDayliteContactRecord(raw)).toBe(false);
    });
  });

  describe("mapDayliteContactRecordToEmployee", () => {
    it("maps contact urls and attributes to employee fields", () => {
      const employee = mapDayliteContactRecordToEmployee({
        self: "/v1/contacts/1000",
        first_name: "Thomas",
        middle_name: "Michael",
        last_name: "Bartelmess",
        keywords: ["Monteur", "Elektrik"],
        addresses: [
          {
            city: "Köln",
            country: "Deutschland",
          },
        ],
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
      });

      expect(employee).toEqual({
        id: "1000",
        dayliteReference: "/v1/contacts/1000",
        name: "Thomas Michael Bartelmess",
        skills: ["Monteur", "Elektrik"],
        homeLocation: "Köln, Deutschland",
        primaryIcalUrl: "https://example.com/primary.ics",
        absenceIcalUrl: "https://example.com/absence.ics",
        active: true,
      });
    });

    it("falls back to extra_fields for ical urls", () => {
      const employee = mapDayliteContactRecordToEmployee({
        self: "/v1/contacts/2000",
        first_name: "Anna",
        last_name: "Schmidt",
        extra_fields:
          '{"primary_ical_url":{"value":"https://example.com/primary.ics"},"absence_ical_url":{"value":"https://example.com/absence.ics"}}',
      });

      expect(employee.primaryIcalUrl).toBe("https://example.com/primary.ics");
      expect(employee.absenceIcalUrl).toBe("https://example.com/absence.ics");
    });
  });

  describe("isAssignment", () => {
    it("validates assignment shape and enums", () => {
      const assignment: unknown = {
        id: "asg-1",
        employeeId: "emp-1",
        projectId: "proj-1",
        period: {
          startDate: "2026-01-26",
          endDate: "2026-01-26",
        },
        source: "manual",
        syncStatus: "synced",
      };

      expect(isAssignment(assignment)).toBe(true);
      expect(isAssignment({ ...assignment, source: "other" })).toBe(false);
    });
  });

  describe("isSyncIssue", () => {
    it("validates sync issue shape and source", () => {
      const issue: unknown = {
        source: "daylite",
        code: "rate_limit",
        message: "Too many requests",
        timestamp: "2026-02-13T12:00:00.000Z",
      };

      expect(isSyncIssue(issue)).toBe(true);
      expect(isSyncIssue({ ...issue, source: "other" })).toBe(false);
    });
  });
});
