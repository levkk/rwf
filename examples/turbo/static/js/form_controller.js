import { Controller } from 'hotwired/stimulus'; // see header.html for importmap

export default class extends Controller {
    connect() {
        window.scrollTo(0, document.body.scrollHeight);
    }

    typingStart() {
        clearTimeout(this.typingStartTimeout);

        this.typingStartTimeout = setTimeout(() => {
            this.typing(true);

            clearTimeout(this.typingStopTimeout);
            this.typingStopTimeout = setTimeout(() => {
                this.typing(false)
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