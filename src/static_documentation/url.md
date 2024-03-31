### URL assert

Check the last fetched URL. This is most meaningful if you have told Hurl to follow redirection (see [`[Options]`section][options] or
[`--location` option]). URL assert consists of the keyword `url` followed by a predicate function and value.

```hurl
GET https://example.org/redirecting
[Options]
location: true
HTTP 200
[Asserts]
url == "https://example.org/redirected"
```