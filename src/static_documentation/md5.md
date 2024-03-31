### MD5 assert

Check response body [MD5] hash.

```hurl
GET https://example.org/data.tar.gz
HTTP 200
[Asserts]
md5 == hex,ed076287532e86365e841e92bfc50d8c;
```