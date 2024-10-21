def application(env, start_response):
    print(env)
    body = [b'Hello']
    start_response('200 OK', [('Content-Type', 'text/plain'), ('Content-Length', str(len(body)))])
    return body
