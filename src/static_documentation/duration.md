### Duration assert

Check the total duration (sending plus receiving time) of the HTTP transaction.

```hurl
GET https://sample.org/helloworld
HTTP 200
[Asserts]
duration < 1000   # Check that response time is less than one second
```
