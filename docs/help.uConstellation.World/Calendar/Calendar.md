---
aliases:
  - Calendar Panel
  - Daily Notes Calendar
description: A monthly calendar view integrated with daily notes and task due dates.
---

# Calendar

The Calendar panel in the right sidebar provides a monthly calendar view that shows which days have notes and tasks, with one-click daily note creation.

## Using the Calendar

Open the Calendar tab in the right sidebar (the calendar icon 📅 in the tab bar).

### Navigation

| Action | How |
|--------|-----|
| Previous month | Click the **‹** arrow |
| Next month | Click the **›** arrow |
| Jump to today | Click the month/year header |

### Day Indicators

Each day cell shows dot indicators:

| Dot Color | Meaning |
|-----------|---------|
| Purple | Notes exist for this day (modified or with `date` frontmatter property) |
| Red | Tasks are due on this day |

The **current day** is highlighted with a purple circle.

### Creating Daily Notes

Click any day to open or create a daily note for that date:
- If a daily note already exists, it opens in a new tab
- If no daily note exists, one is created with a `date` frontmatter property

> [!tip]
> The daily note format and folder can be configured in Settings. The default format is `YYYY-MM-DD`.

## Data Sources

The calendar aggregates dates from two sources:

1. **File dates**: The modification date of each `.md` file, plus any `date` or `created` frontmatter properties
2. **Task due dates**: Due dates from incomplete tasks across all vaults

## Integration with Tasks

Days with overdue tasks show red dots, making it easy to spot missed deadlines at a glance. Click a day to see what tasks are due by opening the daily note.

> [!tip] RTL Support
> The calendar grid respects the current text direction. In RTL languages (Arabic, Hebrew, Farsi, Urdu), the calendar layout adjusts accordingly.
