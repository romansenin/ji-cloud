# Fastly CDN setup

Used for all static assets.

Make sure to set interconnect location that's closest to storage (e.g. Amsterdam Fastly for Belgium Google)

Only each origin should have host request condition, in order for that origin to be used for the domain. e.g. `req.http.host == "docs.jicloud.org"`

A small `VCL Snippet` for the `recv` block is required to make it fetch index.html for plain directory requests:

```
if (req.url ~ "\/$") {
  set req.url = req.url "index.html";
}
```

See Fastly documentation for more details

# Purging

Some buckets are purged automatically via a google cloud function (see [google cloud](../google_cloud/google_cloud.md) on every file change

Others are not and require manual purging, either because they are never expected to be purged (i.e. uploads) or it would depend on the exact route and very rarely need to be purged (i.e. page templates)

For buckets that are purged automatically, the file's cache-control headers are set to not cache, as per Fastly's recommendation. This setting is done immediately before the file is purged