import { Controller } from "hotwired/stimuls";

export default class extends Controller {
  reload() {
    Turbo.visit(window.location.pathname, { action: "replace" });
  }
}
