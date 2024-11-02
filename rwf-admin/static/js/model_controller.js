import { Controller } from "hotwired/stimulus";

export default class extends Controller {
  connect() {
    const elems = this.element.querySelectorAll("select");
    M.FormSelect.init(elems);
    M.updateTextFields();
  }
}
