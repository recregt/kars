# Authentication & Access Control

KARS does **not** ship with a built-in authentication/authorization layer.

If you run KARS on a public IP/domain, you must place it behind your own access control solution. This document uses **Cloudflare Zero Trust / Access** as the default recommendation and then lists alternatives.

---

## Recommended Default: Cloudflare Zero Trust / Access

Cloudflare Access protects your app before requests reach your server. Users must authenticate with an identity provider (Google, GitHub, Microsoft, etc.) and satisfy policy rules.

### What you need

- A domain managed in Cloudflare DNS
- KARS running on a server (for example `http://127.0.0.1:3001`)
- A Cloudflare account with Zero Trust enabled

### 1) Add your application in Cloudflare Access

1. Open Cloudflare Dashboard → **Zero Trust** → **Access** → **Applications**.
2. Click **Add an application**.
3. Choose **Self-hosted**.
4. Configure:
   - **Application name:** `KARS`
   - **Domain:** e.g. `kars.example.com`
   - **Session duration:** choose based on your risk profile (e.g. 24h)

### 2) Configure identity providers

In Zero Trust dashboard:

1. Go to **Settings** → **Authentication**.
2. Add one or more IdPs (Google Workspace, GitHub, Microsoft Entra ID, Okta, etc.).
3. Verify login works for your tenant/users.

### 3) Create Access policy

Create at least one **Allow** policy, for example:

- Include emails ending with `@your-company.com`
- Or include a specific group from your IdP

Optional hardening:

- Require MFA in IdP
- Restrict by country
- Restrict by device posture (Cloudflare WARP posture checks)

### 4) Route traffic to your KARS server

You can use one of these Cloudflare patterns:

- **Cloudflare Tunnel (recommended):** no inbound public port needed on your VM
- **Proxied DNS + reverse proxy:** keep your existing Nginx/Caddy setup

If using Tunnel, install and run `cloudflared` on the server and map hostname to KARS origin (`http://localhost:3001`).

### 5) Keep KARS origin private

Best practice is to avoid exposing `:3001` publicly.

- Bind KARS to localhost or private interface where possible
- Allow inbound traffic only from your reverse proxy/tunnel process
- Close direct public access to origin port in firewall/security group

### 6) Verify enforcement

After setup:

1. Open `https://kars.example.com` in an incognito window.
2. Confirm Access login screen appears.
3. Authenticate with an allowed identity.
4. Confirm blocked identities cannot reach KARS.

### 7) Operations checklist

- Review Access logs regularly
- Keep emergency admin account policy documented
- Rotate credentials/tokens used by tunnel or reverse proxy
- Re-check policies after team/domain changes

---

## Alternative auth layers (brief)

If you do not use Cloudflare, you can protect KARS with any edge auth gateway/reverse proxy solution.

- **Authelia + reverse proxy (Nginx/Caddy/Traefik)**
  - Docs: https://www.authelia.com/integration/proxies/introduction/
- **Authentik as identity gateway**
  - Docs: https://docs.goauthentik.io/docs/add-secure-apps/providers/proxy/
- **OAuth2-Proxy (with Google/GitHub/OIDC)**
  - Docs: https://oauth2-proxy.github.io/oauth2-proxy/
- **Caddy Security (plugin-based auth for Caddy)**
  - Docs: https://authcrunch.com/docs/caddy-security/
- **Traefik ForwardAuth (with external auth service)**
  - Docs: https://doc.traefik.io/traefik/middlewares/http/forwardauth/
- **Tailscale / private network only (no public exposure)**
  - Docs: https://tailscale.com/kb

---

## Responsibility reminder

KARS intentionally focuses on media tracking features and does not enforce user auth itself.

Anyone deploying or sharing KARS is responsible for:

- Selecting an auth/access control architecture
- Enforcing least-privilege access
- Operating and auditing that auth layer

If your deployment is public-facing, do not run KARS without a protective auth gateway in front of it.
