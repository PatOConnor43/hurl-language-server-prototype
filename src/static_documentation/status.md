### Status assert

Check the received HTTP response status code. Status assert consists of the keyword `status` followed by a predicate
function and value.

```hurl
GET https://example.org
HTTP *
[Asserts]
status < 300
```