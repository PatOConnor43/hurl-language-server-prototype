### Body assert

Check the value of the received HTTP response body when decoded as a string.
Body assert consists of the keyword `body` followed by a predicate function and
value. The encoding used to decode the body is based on the `charset` value in the
`Content-Type` header response.

```hurl
GET https://example.org
HTTP 200
[Asserts]
body contains "<h1>Welcome!</h1>"
```

```hurl
# Our HTML response is encoded with GB 2312 (see https://en.wikipedia.org/wiki/GB_2312)
GET https://example.org/cn
HTTP 200
[Asserts]
header "Content-Type" == "text/html; charset=gb2312"
bytes contains hex,c4e3bac3cac0bde7; # 你好世界 encoded in GB 2312
body contains "你好世界"
```

If the `Content-Type` doesn't include any encoding hint, a [`decode` filter] can be used to explicitly decode the body response
bytes.

```hurl
# Our HTML response is encoded using GB 2312.
# But, the 'Content-Type' HTTP response header doesn't precise any charset,
# so we decode explicitly the bytes.
GET https://example.org/cn
HTTP 200
[Asserts]
header "Content-Type" == "text/html"
bytes contains hex,c4e3bac3cac0bde7; # 你好世界 encoded in GB2312
bytes decode "gb2312" contains "你好世界"
```