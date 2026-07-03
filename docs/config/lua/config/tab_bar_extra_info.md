---
tags:
  - tab_bar
  - appearance
---
# Tab Bar Extra Info

When the vertical tab bar is enabled (see [tab_bar_vertical](tab_bar_vertical.md)),
each tab can display extra information below the tab title:

* **Path** - the current working directory (displayed in reversed order)
* **Git Branch** - the current git branch name
* **Current Command** - the currently running process

The extra info panel is configured through `config.colors.tab_bar.extra_info`.

## Default Behavior

By default, all three info items are shown with the following colors:

| Item | Default Color |
|------|---------------|
| Path | Blue (from color palette) |
| Git Branch | Green (from color palette) |
| Current Command | Grey (from color palette) |

## Configuration

```lua
config.colors = {
  tab_bar = {
    extra_info = {
      path = {
        show = true,             -- Whether to show the path
        fg_color = '#6699ff',    -- Foreground color
        -- bg_color = nil,       -- Background color (defaults to tab background)
        -- intensity = 'Normal', -- Text intensity: 'Half', 'Normal', or 'Bold'
        -- italic = false,       -- Whether the text is italic
        -- underline = 'None',   -- Underline style: 'None', 'Single', 'Double',
        --                         --   'Curly', 'Dotted', 'Dashed'
      },
      git_branch = {
        show = true,
        fg_color = '#50fa7b',
        italic = true,
      },
      current_command = {
        show = true,
        fg_color = '#bd93f9',
      },
    },
  },
}
```

## Per-Item Options

Each info item (`path`, `git_branch`, `current_command`) supports the following options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `show` | boolean | `true` | Whether to display this info item |
| `fg_color` | string | *palette color* | Foreground text color (e.g. `'#ff0000'`) |
| `bg_color` | string | *tab background* | Background color |
| `intensity` | string | `'Normal'` | Text intensity: `'Half'`, `'Normal'`, or `'Bold'` |
| `italic` | boolean | `false` | Whether the text is italic |
| `underline` | string | `'None'` | Underline style: `'None'`, `'Single'`, `'Double'`, `'Curly'`, `'Dotted'`, `'Dashed'` |

## Hiding Specific Items

To hide a specific info item, set `show = false`:

```lua
config.colors = {
  tab_bar = {
    extra_info = {
      current_command = { show = false },
    },
  },
}
```

## Notes

* The path is displayed in reversed order (e.g. `wezteam\work\D:`) to match the tab title format.
* The git branch is detected by reading `.git/HEAD` from the current working directory.
* The current command is updated when the tab title changes or on mouse/focus events.
  Process start/exit events do not trigger updates; this is a known limitation.
