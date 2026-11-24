import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";

const target = document.getElementById("app");
if (!target) {
  throw new Error("Crowsnest could not find its root element");
}

export default mount(App, { target });
