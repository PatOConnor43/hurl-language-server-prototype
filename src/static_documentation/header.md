### Header assert

Check the value of a received HTTP response header. Header assert consists of the keyword `header` followed by the value
of the header, a predicate function and a predicate value. Like [headers implicit asserts], the check is 
case-insensitive for the name: comparing a `Content-Type` header is equivalent to a `content-type` one.

```hurl
GET https://example.org
HTTP 302
[Asserts]
header "Location" contains "www.example.net"
header "Last-Modified" matches /\d{2} [a-z-A-Z]{3} \d{4}/
```

If there are multiple headers with the same name, the header assert returns a collection, so `count`, `includes` can be
used in this case to test the header list.

Let's say we have this request and response:

```
> GET /hello HTTP/1.1
> Host: example.org
> Accept: */*
> User-Agent: hurl/2.0.0-SNAPSHOT
>
* Response: (received 12 bytes in 11 ms)
*
< HTTP/1.0 200 OK
< Vary: Content-Type
< Vary: User-Agent
< Content-Type: text/html; charset=utf-8
< Content-Length: 12
< Server: Flask Server
< Date: Fri, 07 Oct 2022 20:53:35 GMT
```

One can use explicit header asserts:

```hurl
GET https://example.org/hello
HTTP 200
[Asserts]
header "Vary" count == 2
header "Vary" includes "User-Agent"
header "Vary" includes "Content-Type"
```

Or implicit header asserts:

```hurl
GET https://example.org/hello
HTTP 200
Vary: User-Agent
Vary: Content-Type
```