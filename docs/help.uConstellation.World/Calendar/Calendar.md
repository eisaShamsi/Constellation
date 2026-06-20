---
aliases:
  - Calendar Panel
  - Daily Notes Calendar
  - Cultural Calendars
description: A full-page month view across eight calendars, with clickable days, daily-note creation, task due dates, and cultural-date recording.
---

# Calendar

The **Calendar** is a full-page month view, opened from the **left dock** (the calendar icon). Days that have notes or due tasks are marked with coloured **dots**. The header shows the month in your chosen calendar; if you set a **secondary calendar**, a subtitle below shows that calendar's equivalent range (for example, a Gregorian month shows its Hijri span, "Dhul-Hijjah 1447 – Muharram 1448 AH").

## Clicking a Day

Every day cell is interactive:

| Action | Result |
|--------|--------|
| Click the empty space (or the day number) | Opens — or creates — that day's **daily note**. Clicking a date that already has a daily note simply **opens** it; it never makes a duplicate. |
| Click a dot | Opens that specific item. If a day has several notes or tasks, clicking the dot shows a small **list** to pick from. |
| Click a task dot | Opens the note **scrolled to that task's line**, ready to edit. |

### Dot Colours

| Dot Colour | Meaning |
|-----------|---------|
| Gold | The **daily note** for that day |
| Purple | Another **note** edited (or dated) that day |
| Red | A **task** due that day |

All dot colours — and every other part of the calendar — are themable in the **Style Setter → Calendar** surface.

> [!tip]
> In the task list you can **tick a task's checkbox to complete it** right from the calendar — completed tasks drop off immediately. Only tasks that carry their own `📅 YYYY-MM-DD` due date appear on the calendar (the date is what places them on a day).

## Cultural Calendars (Eight)

In **Settings → Calendar** you can set the **calendar system**, and the whole month grid switches to it:

- **Gregorian**
- **Hijri (Islamic)** — an accurate astronomical engine; sacred months are highlighted and Islamic events are marked.
- **Solar Hijri (Persian)**
- **Hebrew**
- **Indian (Saka)**
- **Buddhist**
- **Chinese** — *lunisolar*
- **Korean** — *lunisolar*

Each cell shows both the chosen-calendar date (large) and the Gregorian date (small), plus the moon phase. Each month header shows the month **name, its number in parentheses, and the year** — the number helps with calendars whose month order is unfamiliar.

The **Chinese and Korean** calendars are *lunisolar*: they sometimes insert a **leap month** (闰六月 / 윤6월), which the calendar shows as its own page so navigation never skips or doubles it.

You can also choose the **week start** (Sunday/Monday) and toggle the **week-number column**.

### Hijri Calendar Options

Under **Settings → Calendar → "Hijri calendar (Islamic)"** there are two extra controls:

- **Calculation method** — **Astronomical (Lunar Conjunction)**, which follows the true new-moon (most accurate, the default), or **Tabular (al-Tawfīqāt al-Ilhāmiyyah)**, the classical arithmetic cycle.
- **Month correction** — nudge a Hijri month's start by ±1 or ±2 days to match a **local moon sighting**. Pick the Hijri year and month, choose an offset, and click **Set**; the correction applies to that month and every month after it. Your corrections are listed (each removable), with a **Clear all** button.

Both settings (and your corrections) are saved **with your universe**, so they travel across your devices.

### Chinese & Korean Display Options

Korea uses the Chinese lunar calendar, so the two share identical dates — what distinguishes them is the **script** and the **year**. When either is your main or secondary calendar, **Settings → Calendar** shows two extra controls:

- **Year display** — Chinese: the sexagenary cycle 丙午年, the plain year, or both; Korean: the **Dangi** era 단기 4359, the year, or the sexagenary 병오년.
- **Month names** — *native script* (五月 / 5월), or *phonetic* — the month's pronunciation written in your own language (English "Wǔyuè / Owol"; Arabic "وُو-يوي / أوه-وُل").

## Styling the Calendar

Open the **Style Setter** (left dock, or **Settings → Style Setter**) and pick the **Calendar** surface to restyle every part — each element has its own **colour and text size** (day numbers, the cross-reference date, the month pill, weekday headers, week numbers, the moon glyph, the Today highlight, grid lines, and the note/task/event dots), plus the calendar **font**. A live, full-size preview updates as you edit; click **Keep** to apply.

## Daily Notes

The Calendar fully serves daily notes: click any day to open it, or run the **"Daily Note"** command (command palette) to jump to today.

> [!tip]
> **Daily-note filenames always stay Gregorian** (`YYYY-MM-DD`) regardless of the displayed calendar — so your files stay portable and sort correctly. The cultural date is shown in the calendar, and can be recorded in the note's frontmatter (below).

## Recording a Cultural Date in a Note

Two opt-in tools write the cultural date into a note's **properties** (the filename always stays Gregorian `YYYY-MM-DD`):

- **Daily-note Hijri stamp** — *Settings → Calendar → "Stamp the Hijri date in daily notes."* When on (available only while the Hijri calendar is your **main or secondary**), every **new** daily note gets a `hijri:` line, for example `hijri: 1448-01-06`. Notes you already have are never touched.
- **"+ Hijri" in a note's Properties** — open any note's **Properties**, hover the date, and a small **"+ Hijri"** button appears (plus "+ Jalali", "+ Hebrew", and so on — **one button per non-Gregorian calendar you've selected**). Click it and Constellation reads the note's Gregorian date and adds the equivalent, for example `jalali: 1405-03-30`. The Korean button writes the **Dangi** year; a Chinese/Korean **leap month** is marked with an `L` (for example `chinese: 2025-06L-17`). If the note has no date property, it uses the file's creation date.

> [!tip] RTL Support
> The calendar grid respects the current text direction. In RTL languages (Arabic, Hebrew, Farsi, Urdu), the calendar layout adjusts accordingly.
