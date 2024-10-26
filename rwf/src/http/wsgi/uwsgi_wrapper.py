
def wrapper(env, application):
    # List is passed by reference into the lambda function.
    results = []

    # Start response for wsgi.
    def start_response(code, headers):
        results.append(code)
        results.append(headers)

    response = application(env, start_response)

    return (response, results[0], results[1])
