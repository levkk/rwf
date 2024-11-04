import { Controller } from "hotwired/stimulus";

export default class extends Controller {
  reload() {
    Turbo.visit(window.location.href, { action: "replace" });
  }
}
