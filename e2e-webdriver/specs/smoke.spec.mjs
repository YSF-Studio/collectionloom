describe("CollectionLoom Tauri shell", () => {
  it("shows the sidebar and app title", async () => {
    await browser.waitUntil(
      async () => (await $$(".sidebar-item")).length > 0,
      { timeout: 30000, timeoutMsg: "sidebar items did not appear" },
    );
    const title = await $(".titlebar .title");
    await title.waitForDisplayed({ timeout: 10000 });
    expect(await title.getText()).toMatch(/CollectionLoom/i);
  });

  it("navigates core sidebar tabs without crashing", async () => {
    const tabs = ["Disk Imaging", "RAM Capture", "Prerequisites", "About"];
    for (const label of tabs) {
      const item = await $(`.sidebar-item=${label}`);
      await item.waitForClickable({ timeout: 10000 });
      await item.click();
      await browser.pause(600);
      const shell = await $(".app-shell");
      expect(await shell.isDisplayed()).toBe(true);
    }
  });
});
