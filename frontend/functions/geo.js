// Runs on Cloudflare's edge network where the visitor's country is known.
// Free, no external geo-IP service needed.
export function onRequest(context) {
  const country =
    context.request.cf?.country ||
    context.request.headers.get("CF-IPCountry") ||
    "CA";
  return new Response(JSON.stringify({ country }), {
    headers: {
      "content-type": "application/json",
      "cache-control": "no-store",
    },
  });
}