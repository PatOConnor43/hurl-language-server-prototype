### Cookie assert

Check value or attributes of a [`Set-Cookie`] response header. Cookie assert
consists of the keyword `cookie`, followed by the cookie name (and optionally a
cookie attribute), a predicate function and value.

Cookie attributes value can be checked by using the following format:
`<cookie-name>[cookie-attribute]`. The following attributes are supported: `Value`,
`Expires`, `Max-Age`, `Domain`, `Path`, `Secure`, `HttpOnly` and `SameSite`.

```hurl
GET http://localhost:8000/cookies/set
HTTP 200

# Explicit check of Set-Cookie header value. If the attributes are
# not in this exact order, this assert will fail. 
Set-Cookie: LSID=DQAAAKEaem_vYg; Expires=Wed, 13 Jan 2021 22:23:01 GMT; Secure; HttpOnly; Path=/accounts; SameSite=Lax;
Set-Cookie: HSID=AYQEVnDKrdst; Domain=localhost; Expires=Wed, 13 Jan 2021 22:23:01 GMT; HttpOnly; Path=/
Set-Cookie: SSID=Ap4PGTEq; Domain=localhost; Expires=Wed, 13 Jan 2021 22:23:01 GMT; Secure; HttpOnly; Path=/

# Using cookie assert, one can check cookie value and various attributes.
[Asserts]
cookie "LSID" == "DQAAAKEaem_vYg"
cookie "LSID[Value]" == "DQAAAKEaem_vYg"
cookie "LSID[Expires]" exists
cookie "LSID[Expires]" contains "Wed, 13 Jan 2021"
cookie "LSID[Max-Age]" not exists
cookie "LSID[Domain]" not exists
cookie "LSID[Path]" == "/accounts"
cookie "LSID[Secure]" exists
cookie "LSID[HttpOnly]" exists
cookie "LSID[SameSite]" equals "Lax"
```

> `Secure` and `HttpOnly` attributes can only be tested with `exists` or `not exists` predicates
> to reflect the [Set-Cookie header] semantics (in other words, queries `<cookie-name>[HttpOnly]`
> and `<cookie-name>[Secure]` don't return boolean).