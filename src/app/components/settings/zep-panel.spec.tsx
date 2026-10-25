import { describe, expect, it, mock } from "bun:test";
import { screen } from "@testing-library/react";
import type { ZepCredentialsInfo } from "../../../generated/tauri";
import { commandFailed, commandOk } from "../../../test/app-error";
import { mockCommands } from "../../../test/mock-commands";
import { renderComponent } from "../../../test/render-component";
import { ZepSettingsPanel } from "./zep-panel";

const mockLoadCredentials = mock(() =>
  commandOk<ZepCredentialsInfo | null>({
    rootUrl: "https://app.zep.de/caldav/admin",
    username: "planer",
  }),
);

mockCommands({ zepLoadCredentials: mockLoadCredentials });

describe("ZepSettingsPanel", () => {
  it("fills in the stored URL and username", async () => {
    renderComponent(<ZepSettingsPanel onClose={() => {}} />);

    expect(
      await screen.findByDisplayValue("https://app.zep.de/caldav/admin"),
    ).toBeDefined();
    expect(screen.getByDisplayValue("planer")).toBeDefined();
  });

  it("explains in German when the stored credentials cannot be read", async () => {
    mockLoadCredentials.mockImplementationOnce(() =>
      commandFailed("Die ZEP-Zugangsdaten konnten nicht geladen werden."),
    );

    renderComponent(<ZepSettingsPanel onClose={() => {}} />);

    expect(
      await screen.findByText(
        "Die ZEP-Zugangsdaten konnten nicht geladen werden.",
      ),
    ).toBeDefined();
  });
});
