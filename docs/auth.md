# Auth plan (Microsoft → Xbox → Minecraft)

Standard third-party-launcher flow (no official Mojang launcher library):

1. Microsoft OAuth2 (device-code flow, public client): scope `XboxLive.signin`.
2. Xbox Live auth (`user.auth.xboxlive.com`) → XSTS (`xsts.auth.xboxlive.com`,
   relying party `http://auth.xboxlive.com` for MC).
3. `POST https://api.minecraftservices.com/authentication/login_with_xbox`
   → Minecraft access token.
4. `GET .../minecraft/profile` → UUID + skin; check GamePass/entitlements.
5. Store refresh tokens in OS keychain; refresh silently.

Requires a Microsoft Entra application (client ID). Options: register our own
public-client app ID (needs user decision) or let advanced users supply their
own Azure app credentials. Tokens never leave the machine.
