
response_code = None
response_headers = None

def wrapper(env, application):
    response = application(env, start_response)
    return response

def start_response(code, headers):
    global response_code
    global response_headers
    response_code = code
    response_headers = headers

def get_code():
    return response_code

def get_headers():
    return response_headers
