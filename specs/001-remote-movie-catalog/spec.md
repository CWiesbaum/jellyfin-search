# Feature Specification: Remote Movie Library Overview

**Feature Branch**: `001-remote-movie-catalog`

**Created**: 2026-09-16

**Status**: Draft

**Input**: User description: "Background: The movie library on the home media server has grown large enough that, while out and about, it's no longer possible to keep track of which movies are already available. The media server itself must not be directly reachable from the internet, for security reasons. Goal: A browsable, easily searchable overview of the existing movie library that is reachable at any time while away from home, so it can be quickly checked whether a specific movie is already available, without requiring a connection to the media server or the home network. Functional requirements: Display of all movies currently in the library (at minimum title, ideally also release year, genre, and thumbnail image); Ability to search or filter the list by title; The displayed data must be refreshed regularly, so that newly added movies become visible in a timely manner (some delay compared to the actual library state is acceptable); The overview must be usable while away from home, i.e. outside the home network, without requiring an additional connection setup (e.g. VPN); Access to the overview must not enable or require direct access to the media server itself. Non-goals: No playback/streaming of movies through this overview — inventory lookup only; No management functions (adding, deleting, editing metadata); Not a replacement for the actual Jellyfin interface on the home network. Open question: Should the overview be publicly viewable, or protected by access control (e.g. a password)?"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Check whether a specific movie is available (Priority: P1)

While out and about (e.g., at a store, at a friend's place, or browsing recommendations), a
user wants to know whether a particular movie is already in the home library, so they can
decide whether it's worth adding or looking for elsewhere.

**Why this priority**: This is the core problem statement — the library has grown too large to
remember, and the primary trigger for opening the overview is "is *this* movie already
there?". Without this, the feature delivers no value.

**Independent Test**: Can be fully tested by opening the overview away from the home network,
typing a movie title into the search/filter field, and confirming the correct match (or "no
match") appears without any connection to the home network or media server.

**Acceptance Scenarios**:

1. **Given** the overview is open and the user is outside the home network, **When** the user
   types a movie title (or part of one) into the search field, **Then** matching movies
   already in the library are shown, and titles not present in the library are not shown.
2. **Given** a movie the user searches for is not in the library, **When** the search
   completes, **Then** the overview clearly indicates no match was found (rather than showing
   a blank or ambiguous result).
3. **Given** the user has no VPN or other connection to the home network active, **When** they
   open the overview, **Then** it loads and is fully usable.

---

### User Story 2 - Browse the full library at a glance (Priority: P2)

A user wants to casually scroll through the movie library to remind themselves what's
available, without a specific title in mind — for example, to decide what to watch once they
get home.

**Why this priority**: Complements search with open-ended discovery, and is the scenario that
benefits most from richer metadata (year, genre, thumbnail), but the feature is still useful
(via User Story 1 alone) without it.

**Independent Test**: Can be fully tested by opening the overview and scrolling/paging through
the full movie list without performing a search, confirming every currently-owned movie
appears with its available details.

**Acceptance Scenarios**:

1. **Given** the overview is open, **When** the user browses without entering a search term,
   **Then** all movies currently in the library are listed.
2. **Given** a movie has release year, genre, and thumbnail metadata available, **When** it is
   shown in the list, **Then** that metadata is displayed alongside the title.
3. **Given** a movie is missing some metadata (e.g., no thumbnail), **When** it is shown in the
   list, **Then** it still displays correctly with the available fields, without errors or
   broken layout.

---

### User Story 3 - Trust that the overview reflects recent additions (Priority: P3)

A user who recently added movies to the home library (or knows movies were added) wants
confidence that a movie missing from the overview today might simply not be synced yet, not
necessarily absent from the library.

**Why this priority**: Freshness is what makes the overview trustworthy for decision-making;
without it, users may falsely conclude a movie is missing and duplicate effort (e.g., buying a
movie that's already available). Ranked P3 because the overview still delivers core value even
with a known, bounded sync delay.

**Independent Test**: Can be fully tested by adding a movie to the home library, waiting for
the next scheduled refresh, and confirming it becomes visible in the overview without any
manual action.

**Acceptance Scenarios**:

1. **Given** a new movie was added to the home library, **When** the next scheduled data
   refresh runs, **Then** the movie appears in the overview without any manual trigger.
2. **Given** the overview is open, **When** the user looks at it, **Then** they can tell how
   recent the displayed data is (e.g., a "last updated" indication).

---

### Edge Cases

- What happens when the home media server or home internet connection is unreachable at the
  time a scheduled refresh would normally run? The overview MUST continue showing the last
  successfully synced data rather than an error or empty list, and refreshing MUST resume
  automatically once connectivity is restored.
- What happens when a movie is removed from the library at home? It MUST no longer appear in
  the overview after the next successful refresh.
- How does the system handle a search with no matching results? It MUST clearly communicate
  "not found" rather than showing nothing with no explanation.
- How does the system handle movies with incomplete metadata (missing year, genre, or
  thumbnail)? The movie MUST still display correctly using a placeholder for the missing
  field(s).
- The overview has no concept of "unauthorized" access — per FR-010, it is intentionally open
  to anyone who has its address.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display all movies currently present in the home movie library, at
  minimum showing each movie's title.
- **FR-002**: System MUST display, for each movie where the data is available, the release
  year, genre, and a thumbnail/poster image, in addition to the title.
- **FR-003**: Users MUST be able to search or filter the displayed movie list by title.
- **FR-004**: System MUST automatically refresh the displayed movie data from the home library
  on a recurring schedule, without requiring manual action by the user.
- **FR-005**: The overview MUST be reachable and fully usable by the user while outside the
  home network, without the user establishing any additional connection (such as a VPN) to the
  home network first.
- **FR-006**: The overview MUST NOT require or provide any direct network path from the
  overview's users to the home media server; the home media server MUST remain unreachable
  from the internet.
- **FR-007**: The overview MUST be read-only for inventory-lookup purposes; it MUST NOT provide
  movie playback or streaming.
- **FR-008**: The overview MUST NOT provide library management functions (adding, deleting, or
  editing movie entries or metadata).
- **FR-009**: When the home media server or home internet connection is temporarily
  unreachable at a scheduled refresh time, the overview MUST continue serving the last
  successfully synced data, and MUST resume refreshing automatically once connectivity returns.
- **FR-010**: The overview MUST be publicly viewable by anyone who has its address, without
  requiring a login, password, or any other authentication step.

### Key Entities

- **Movie Entry**: A single movie as shown in the overview. Attributes: title, release year
  (optional), genre(s) (optional), thumbnail/poster image (optional), and presence in the home
  library as of the last sync.
- **Catalog Snapshot**: The independently accessible copy of the movie library's data that the
  overview displays, since it cannot query the home media server directly. Attributes: the set
  of current movie entries, and a last-updated/last-synced timestamp shown to the user.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user away from home can determine whether a specific movie title is already in
  the library in under 30 seconds from opening the overview.
- **SC-002**: A movie newly added to the home library becomes visible in the overview within 24
  hours, without any manual action by the user.
- **SC-003**: The overview remains reachable and usable from outside the home network 100% of
  the time that its own hosting is available — independent of whether the home internet
  connection or home media server is currently online.
- **SC-004**: Users can locate a specific movie via search in under 10 seconds, even as the
  library grows to several thousand titles.
- **SC-005**: Zero instances of the overview exposing a direct network path to the home media
  server (verified by design/security review, not user-facing behavior).

## Assumptions

- "Movies" refers specifically to the movie library; other library types (TV shows, music,
  etc.) are out of scope for this feature unless a future feature extends it.
- The home media server can provide, or be made to provide, an export/feed of the movie
  library's metadata (title, year, genre, thumbnail) needed to populate the overview; how that
  export happens is a technical/design concern outside this spec.
- Because the home media server must stay unreachable from the internet, the overview is
  necessarily a separate, independently hosted surface that holds a periodically synced copy
  of the catalog data, rather than a live view into the media server.
- "Refreshed regularly" is interpreted as an automated, scheduled sync process; up to a 24-hour
  delay between a movie being added at home and it appearing in the overview is acceptable, per
  the stated tolerance for "some delay."
- The overview is intentionally open (no login) per FR-010; it is treated as non-sensitive
  inventory information, since it exposes only movie titles/metadata and no personal, account,
  or media-server access details.
