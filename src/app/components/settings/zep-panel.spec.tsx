import { beforeEach, describe, expect, it, mock } from "bun:test";
import { fireEvent, screen } from "@testing-library/react";
import type { ZepCredentialsInfo } from "../../../generated/tauri";
import { commandFailed, commandOk } from "../../../test/app-error";
import { mockCommands } from "../../../test/mock-commands";
import { renderComponent } from "../../../test/render-component";
import { ZepSettingsPanel } from "./zep-panel";

const initialCredentials: ZepCredentialsInfo = {
  rootUrl: "https://app.zep.de/caldav/admin",
  username: "planer",
};
let storedCredentials = initialCredentials;

const mockLoadCredentials = mock(() =>
  commandOk<ZepCredentialsInfo | null>(storedCredentials),
);

mockCommands({
  zepLoadCredentials: mockLoadCredentials,
  zepTestCredentials: () => commandOk({ calendarCount: 3 }),
  zepSaveCredentials: (rootUrl: string, username: string) => {
    storedCredentials = { rootUrl, username };
    return commandOk(null);
  },
});

describe("ZepSettingsPanel", () => {
  beforeEach(() => {
    storedCredentials = initialCredentials;
  });

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

  it("shows the saved values when the panel is opened again", async () => {
    const { rerender } = renderComponent(
      <ZepSettingsPanel onClose={() => {}} />,
    );
    fireEvent.change(await screen.findByDisplayValue("planer"), {
      target: { value: "disponent" },
    });
    fireEvent.change(screen.getByPlaceholderText("ZEP-Passwort"), {
      target: { value: "geheim" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Verbindung testen & speichern" }),
    );
    await screen.findByText(
      "ZEP-Verbindung erfolgreich gespeichert. 3 Kalender gefunden.",
    );

    rerender(<ZepSettingsPanel key="reopened" onClose={() => {}} />);

    expect(await screen.findByDisplayValue("disponent")).toBeDefined();
  });
});
