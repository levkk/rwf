import { Application } from 'hotwired/stimulus'
import FormController from '/static/js/form_controller.js'

const application = Application.start()
application.register('form', FormController)