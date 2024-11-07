class TodoController < ApplicationController
  skip_before_action :verify_authenticity_token

  def create
    puts params
    puts params[:hello]
    redirect_to "/"
  end
end
