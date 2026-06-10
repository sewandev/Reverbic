# Legal Notices

## Third-Party Services

### Radio Browser API

Radio station metadata and stream URLs are provided by the [Radio Browser](https://www.radio-browser.info/) community API.  
Radio Browser is a free, open, community-maintained database of internet radio stations.  
Reverbic does not host, own, or operate any radio station. All streams are external URLs played directly by the user's machine.

### Spotify

Spotify integration in Reverbic uses the official [Spotify Web API](https://developer.spotify.com/documentation/web-api) for search and playback control, and [librespot](https://github.com/librespot-org/librespot) for audio streaming.

**librespot** is an open-source Spotify client library licensed under the MIT License.  
The librespot project is independent of Spotify AB and is not affiliated with or endorsed by Spotify.

#### Regarding Spotify Terms of Service

- Reverbic uses the **official Spotify OAuth PKCE flow** with a user-supplied Client ID registered in the Spotify Developer Dashboard. Each user is responsible for registering their own application and complying with the [Spotify Developer Terms of Service](https://developer.spotify.com/terms).
- librespot implements the Spotify Connect protocol for private, non-commercial use. Its use is consistent with the [librespot project's own legal position](https://github.com/librespot-org/librespot#disclaimer).
- Reverbic does not facilitate any form of Spotify account sharing, circumvention of DRM, or redistribution of Spotify content.
- **Spotify Premium is required** for audio streaming via librespot. Free-tier accounts cannot use the streaming feature.
- This software is provided for personal, non-commercial use only.

#### Risk of Native Playback Restrictions (2026 Policy Changes)

Spotify's 2026 policy updates around Widevine DRM enforcement and stricter API access (Developer Mode quotas, anti-scraping and anti-AI-misuse measures) target unofficial Spotify Connect clients like librespot and could change at any time without notice.

- **Native playback risk**: Spotify could block or degrade librespot-based audio streaming for accounts that use it, up to and including temporary restrictions on the affected account. This risk applies only to native playback, not to the account itself in normal use.
- **Remote Control mode**: search, playback control, and device transfer via the official Spotify Web API do not depend on librespot, so they are not subject to the native playback risk above. Remote Control mode still depends on the Spotify Developer Mode application that each user registers under their own account (see above) — Spotify's Developer Mode terms (including the Premium requirement for the app owner) can change at any time, so this is a reasonable fallback for the native playback risk specifically, not a permanent guarantee.
- Users who prefer to avoid this risk entirely can use Reverbic exclusively in Remote Control mode, controlling playback on another official Spotify client (desktop, mobile, web).

**References:**
- [Spotify Developer Policy](https://developer.spotify.com/policy)
- [Spotify Web Player Help (DRM/Widevine)](https://support.spotify.com/us/article/web-player-help/)
- [Update on Developer Access and Platform Security (Feb 2026)](https://developer.spotify.com/blog/2026-02-06-update-on-developer-access-and-platform-security)

## Disclaimer

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND. THE AUTHORS ARE NOT
RESPONSIBLE FOR ANY MISUSE OF THIS SOFTWARE OR FOR VIOLATIONS OF THIRD-PARTY TERMS OF
SERVICE BY END USERS. USE OF THIS SOFTWARE IS AT YOUR OWN RISK.
