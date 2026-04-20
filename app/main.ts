import App from "./App.svelte";
import { mount } from "svelte";
import { i18n } from "$lib/i18n";
import { startupMark } from "$lib/utils/startup-timing";
import "./app.css";

startupMark("frontend.main.start");
i18n.init();
startupMark("frontend.main.i18n-ready");

const app = mount(App, {
  target: document.getElementById("app")!,
});
startupMark("frontend.main.mounted");

export default app;
