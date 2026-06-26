local has_telescope, telescope = pcall(require, "telescope")
if not has_telescope then return end

local actions = require("telescope.actions")
local action_state = require("telescope.actions.state")
local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local conf = require("telescope.config").values
local sm = require("sm")

local function task_picker(title, cmd, keymap_defs)
  return function()
    local raw = sm.run_sync(cmd)
    local lines = vim.split(raw, "\n")
    local entries = {}
    for _, line in ipairs(lines) do
      local parsed = sm.parse_line(line)
      if parsed then
        table.insert(entries, parsed)
      end
    end
    pickers.new(_, {
      prompt_title = title,
      finder = finders.new_table {
        results = entries,
        entry_maker = function(entry)
          return {
            value = entry,
            display = entry.description .. "  [" .. entry.tags[1] .. "]",
            ordinal = entry.description .. " " .. entry.raw,
          }
        end,
      },
      sorter = conf.generic_sorter(_),
      attach_mappings = function(prompt_bufnr, map)
        for _, def in ipairs(keymap_defs) do
          map(def.mode or "n", def.key, function()
            local selection = action_state.get_selected_entry()
            if selection and def.fn then
              def.fn(selection.value)
            end
            actions.close(prompt_bufnr)
          end)
        end
        return true
      end,
    }):find()
  end
end

local function menu_entry(text, fn)
  return { value = fn, display = text, ordinal = text }
end

local function action_picker(title, items)
  pickers.new(_, {
    prompt_title = title,
    finder = finders.new_table {
      results = items,
      entry_maker = function(entry)
        return { value = entry.value, display = entry.display, ordinal = entry.ordinal }
      end,
    },
    sorter = conf.generic_sorter(_),
    attach_mappings = function(prompt_bufnr, map)
      map("n", "<CR>", function()
        local selection = action_state.get_selected_entry()
        if selection and selection.value then
          actions.close(prompt_bufnr)
          selection.value()
        end
      end)
      map("i", "<CR>", function()
        local selection = action_state.get_selected_entry()
        if selection and selection.value then
          actions.close(prompt_bufnr)
          selection.value()
        end
      end)
      return true
    end,
  }):find()
end

local function prompt_add()
  vim.ui.input({ prompt = "Task description: " }, function(input)
    if input and #input > 0 then
      sm.add(input, {}, false)
      sm.notify("Added to backlog: " .. input)
    end
  end)
end

local function prompt_add_today()
  vim.ui.input({ prompt = "Task description (today): " }, function(input)
    if input and #input > 0 then
      sm.add(input, {}, true)
      sm.notify("Added to daily: " .. input)
    end
  end)
end

local function bullet_picker()
  action_picker("SM Bullet", {
    menu_entry("migrate (>)", function()
      vim.ui.input({ prompt = "Task ID: " }, function(id)
        if id then sm.run_sync("bullet --id " .. id .. " --type migrate"); sm.notify("Migrated #" .. id) end
      end)
    end),
    menu_entry("schedule (<)", function()
      vim.ui.input({ prompt = "Task ID: " }, function(id)
        if id then sm.run_sync("bullet --id " .. id .. " --type schedule"); sm.notify("Scheduled #" .. id) end
      end)
    end),
    menu_entry("event (o)", function()
      vim.ui.input({ prompt = "Task ID: " }, function(id)
        if id then sm.run_sync("bullet --id " .. id .. " --type event"); sm.notify("Event #" .. id) end
      end)
    end),
    menu_entry("note (-)", function()
      vim.ui.input({ prompt = "Task ID: " }, function(id)
        if id then sm.run_sync("bullet --id " .. id .. " --type note"); sm.notify("Note #" .. id) end
      end)
    end),
    menu_entry("priority (*)", function()
      vim.ui.input({ prompt = "Task ID: " }, function(id)
        if id then sm.run_sync("bullet --id " .. id .. " --type priority"); sm.notify("Priority #" .. id) end
      end)
    end),
  })
end

return telescope.register_extension {
  setup = function() end,
  exports = {
    tasks = task_picker("SM Tasks", "list", {
      { key = "<CR>", fn = function(t) sm.toggle(t.id); sm.notify("Toggled: " .. t.description) end },
      { key = "dd",   fn = function(t) sm.remove(t.id); sm.notify("Removed: " .. t.description) end },
      { key = "cc",   fn = function(t) sm.cancel(t.id); sm.notify("Cancelled: " .. t.description) end },
      { key = "b",    fn = function(t)
        actions.close(vim.api.nvim_get_current_buf())
        vim.schedule(function()
          vim.ui.input({ prompt = "Bullet type (migrate/schedule/event/note/priority): " }, function(bt)
            if bt then sm.run_sync("bullet --id " .. t.id .. " --type " .. bt); sm.notify(bt .. " #" .. t.id) end
          end)
        end)
      end },
    }),
    backlog = task_picker("SM Backlog", "backlog", {
      { key = "<CR>", fn = function(t) sm.pull(t.id); sm.notify("Pulled: " .. t.description) end },
      { key = "dd",   fn = function(t) sm.remove(t.id); sm.notify("Removed: " .. t.description) end },
      { key = "cc",   fn = function(t) sm.cancel(t.id); sm.notify("Cancelled: " .. t.description) end },
    }),
    scan = function()
      local rg_cmd = 'rg --no-heading --line-number "(- %[ %]|TODO:|FIXME:|HACK:|XXX:)"'
      local results = vim.fn.systemlist(rg_cmd)
      if #results == 0 then
        sm.notify("No TODOs found", vim.log.levels.INFO)
        return
      end
      local entries = {}
      for _, line in ipairs(results) do
        local file, lnum, text = line:match("^([^:]+):(%d+):(.+)$")
        if file then
          table.insert(entries, {
            filename = file,
            lnum = tonumber(lnum),
            text = text,
          })
        end
      end
      pickers.new(_, {
        prompt_title = "SM Scan",
        finder = finders.new_table {
          results = entries,
          entry_maker = function(entry)
            return {
              value = entry,
              display = entry.text,
              ordinal = entry.text,
              filename = entry.filename,
              lnum = entry.lnum,
            }
          end,
        },
        sorter = conf.generic_sorter(_),
        attach_mappings = function(prompt_bufnr, map)
          map("n", "<CR>", function()
            local selection = action_state.get_selected_entry()
            if selection then
              actions.close(prompt_bufnr)
              vim.cmd("edit +" .. selection.lnum .. " " .. selection.filename)
            end
          end)
          map("i", "<C-i>", function()
            local selection = action_state.get_selected_entry()
            if selection then
              local cleaned = selection.text:gsub("^%s*[-%*] %[ %]%s*", ""):gsub("^%s*TODO:%s*", "")
              local desc = vim.trim(cleaned)
              local source = selection.filename .. ":" .. selection.lnum
              sm.add(desc .. " #todo source:" .. source, {}, false)
              sm.notify("Imported: " .. desc)
              actions.close(prompt_bufnr)
            end
          end)
          return true
        end,
      }):find()
    end,
    menu = function()
      action_picker("SM Menu", {
        menu_entry("  Tasks (daily)", function()
          require("telescope").extensions.sm.tasks()
        end),
        menu_entry("  Backlog", function()
          require("telescope").extensions.sm.backlog()
        end),
        menu_entry("󰩈  Add task (backlog)", prompt_add),
        menu_entry("󰩈  Add task (today)", prompt_add_today),
        menu_entry("  Bullet type", bullet_picker),
        menu_entry("  Pomodoro start", function() sm.pomo_start(); sm.notify("▶ Pomodoro started!") end),
        menu_entry("  Pomodoro stop", function() sm.pomo_stop(); sm.notify("⏹ Pomodoro stopped.") end),
        menu_entry("  Pomodoro status", function() sm.notify(sm.pomo_status()) end),
        menu_entry("󰈞  Scan TODOs", function()
          require("telescope").extensions.sm.scan()
        end),
      })
    end,
  },
}
