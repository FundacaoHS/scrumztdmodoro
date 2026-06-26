local has_telescope, telescope = pcall(require, "telescope")
if not has_telescope then return end

local actions = require("telescope.actions")
local action_state = require("telescope.actions.state")
local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local conf = require("telescope.config").values
local sm = require("sm")

local function picker(title, cmd, keymap_defs)
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
            display = entry.description,
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

return telescope.register_extension {
  setup = function() end,
  exports = {
    tasks = picker("SM Tasks", "list", {
      { key = "<CR>", fn = function(t) sm.toggle(t.id); sm.notify("Toggled: " .. t.description) end },
      { key = "dd",   fn = function(t) sm.remove(t.id); sm.notify("Removed: " .. t.description) end },
      { key = "cc",   fn = function(t) sm.cancel(t.id); sm.notify("Cancelled: " .. t.description) end },
    }),
    backlog = picker("SM Backlog", "backlog", {
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
  },
}
