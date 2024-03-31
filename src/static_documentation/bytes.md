### Bytes assert

Check the value of the received HTTP response body as a bytestream. Body assert
consists of the keyword `bytes` followed by a predicate function and value.

```hurl
GET https://example.org/data.bin
HTTP 200
[Asserts]
bytes startsWith hex,efbbbf;
bytes count == 12424
header "Content-Length" == "12424"
```