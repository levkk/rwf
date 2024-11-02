import { Controller } from "hotwired/stimulus";

export default class extends Controller {
  connect() {
    const elems = this.element.querySelectorAll("select");
    M.FormSelect.init(elems);
    M.updateTextFields();
  }

  selectColumn(e) {
    const options = [...e.currentTarget.querySelectorAll('option')]
    const selected = []
    for (let idx in options) {
      let option = options[idx]
      if (option.selected) {
        selected.push(option.value)
      }
    }

    const urlParams = new URLSearchParams(window.location.search);
    urlParams.delete("columns")
    urlParams.append("columns", selected.join(","))
    const url = `${window.location.pathname}?${urlParams.toString()}`

    Turbo.visit(url)
  }
}
