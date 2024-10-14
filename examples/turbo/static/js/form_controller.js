import { Controller } from 'hotwired/stimulus'; // see header.html for importmap

export default class extends Controller {
    connect() {
        window.scrollTo(0, document.body.scrollHeight);
    }

    typingStart() {
      if (this.startedTyping) {
        clearTimeout(this.typingStartTimeout);
      } else {
        this.startedTyping = true;
      }

      this.typingStartTimeout = setTimeout(() => {
        this.typing(true);

        clearTimeout(this.typingStopTimeout);
        this.typingStopTimeout = setTimeout(() => {
          this.typing(false)
          this.startedTyping = false;
        }, 3000)
      }, 300);
    }

    typing(typing) {
        fetch('/chat/typing', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                typing,
            })
        })
    }
}