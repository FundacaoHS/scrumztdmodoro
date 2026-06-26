if vim.fn.executable("sm") == 0 and vim.fn.executable("target/debug/sm.exe") == 0 then
  return
end

local sm = require("sm")

-- List tasks in a scratch buffer
function SM_list()
  local res = sm.run_sync("list")
  if res == "" then res = "no daily tasks" end
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_name(buf, "sm://daily")
  local lines = vim.split(res, "\n")
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_set_current_buf(buf)
  vim.bo[buf].filetype = "sm"
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].modifiable = false
  vim.keymap.set("n", "<CR>", function()
    local line = vim.api.nvim_get_current_line()
    local parsed = sm.parse_line(line)
    if parsed then
      sm.toggle(parsed.id)
      SM_list()
    end
  end, { buffer = buf })
  vim.keymap.set("n", "dd", function()
    local line = vim.api.nvim_get_current_line()
    local parsed = sm.parse_line(line)
    if parsed then
      sm.remove(parsed.id)
      SM_list()
    end
  end, { buffer = buf })
  vim.keymap.set("n", "cc", function()
    local line = vim.api.nvim_get_current_line()
    local parsed = sm.parse_line(line)
    if parsed then
      sm.cancel(parsed.id)
      SM_list()
    end
  end, { buffer = buf })
  vim.keymap.set("n", "q", "<cmd>bdelete<CR>", { buffer = buf })
end

-- Show backlog in scratch buffer
function SM_backlog()
  local res = sm.run_sync("backlog")
  if res == "" then res = "no backlog tasks" end
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_name(buf, "sm://backlog")
  local lines = vim.split(res, "\n")
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_set_current_buf(buf)
  vim.bo[buf].filetype = "sm"
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].modifiable = false
  vim.keymap.set("n", "<CR>", function()
    local line = vim.api.nvim_get_current_line()
    local parsed = sm.parse_line(line)
    if parsed then
      sm.pull(parsed.id)
      SM_backlog()
    end
  end, { buffer = buf })
  vim.keymap.set("n", "q", "<cmd>bdelete<CR>", { buffer = buf })
end

-- Add task
function SM_add(opts)
  local task = opts and opts.args or vim.fn.input("Task: ")
  if task and #task > 0 then
    local result = sm.add(task, {}, opts and opts.bang)
    sm.notify("✓ Task added: " .. task)
  end
end

-- Pomodoro
function SM_pomo(opts)
  local cmd = opts and opts.args or "status"
  if cmd == "start" then
    sm.pomo_start()
    sm.notify("▶ Pomodoro started!")
  elseif cmd == "stop" then
    sm.pomo_stop()
    sm.notify("⏹ Pomodoro stopped.")
  else
    local res = sm.pomo_status()
    sm.notify("Pomodoro: " .. res)
  end
end

-- TODO scanner with rg + fzf
function SM_scan()
  local rg_cmd = 'rg --no-heading --line-number "(- %[ %]|TODO:|FIXME:|HACK:|XXX:)"'
  local fzf_cmd = "fzf --ansi --preview 'bat --color=always {1}:{2}'"
  local cmd = rg_cmd .. " | " .. fzf_cmd

  -- fzf terminal window
  vim.fn.system(cmd)
  -- simpler: just rg and open quickfix
  local results = vim.fn.systemlist(rg_cmd)
  if #results == 0 then
    sm.notify("No TODOs found", vim.log.levels.INFO)
    return
  end
  local qflist = {}
  for _, line in ipairs(results) do
    local file, lnum, text = line:match("^([^:]+):(%d+):(.+)$")
    if file then
      table.insert(qflist, { filename = file, lnum = tonumber(lnum), text = text })
    end
  end
  vim.fn.setqflist(qflist)
  vim.cmd("copen")
  sm.notify("Found " .. #qflist .. " TODOs in project")
end

-- Highlight SM buffer
vim.api.nvim_create_augroup("sm_highlight", { clear = true })
vim.api.nvim_create_autocmd("FileType", {
  pattern = "sm",
  group = "sm_highlight",
  callback = function()
    vim.fn.matchadd("Conceal", "\\[backlog\\]", 10, -1, { conceal = "B" })
    vim.fn.matchadd("Conceal", "\\[todo\\]", 10, -1, { conceal = "T" })
    vim.fn.matchadd("Conceal", "\\[cancelled\\]", 10, -1, { conceal = "X" })
    vim.wo.conceallevel = 2
    -- Colors
    vim.cmd("syntax match smBullet /^[•x><o\\-\\*]/")
    vim.cmd("highlight default smBullet guifg=#e94560")
    vim.cmd("syntax match smCancelled /\\[cancelled\\]/")
    vim.cmd("highlight default smCancelled guifg=#666 gui=strikethrough")
  end,
})

-- Import from quickfix (TODO scanner) as task
function SM_import()
  local qflist = vim.fn.getqflist()
  if #qflist == 0 then
    sm.notify("No items in quickfix list. Run :SMScan first.", vim.log.levels.WARN)
    return
  end
  local function do_import(entry)
    local task_desc = (entry.text or "")
      :gsub("^%s*[-%*] %[ %]%s*", "")
      :gsub("^%s*TODO:%s*", "")
      :gsub("^%s*", ""):gsub("%s*$", "")
    local full_path = entry.filename
    if full_path and not vim.fn.isabsolutepath(full_path) then
      full_path = vim.fn.getcwd() .. "/" .. full_path
    end
    local source = full_path and vim.fn.fnamemodify(full_path, ":~:.") or "unknown"
    local desc = task_desc .. " #todo source:" .. source .. ":" .. (entry.lnum or "?")
    sm.add(desc, {}, false)
    sm.notify("✓ Imported: " .. task_desc)
  end
  if #qflist == 1 then
    do_import(qflist[1])
    return
  end
  local items = {}
  for i, entry in ipairs(qflist) do
    table.insert(items, {
      label = string.format("%s:%d %s", vim.fn.fnamemodify(entry.filename, ":t") or "?", entry.lnum or 0,
        (entry.text or ""):sub(1, 60)),
      entry = entry,
    })
  end
  vim.ui.select(items, {
    prompt = "Import TODO as task:",
    format_item = function(item) return item.label end,
  }, function(choice)
    if choice then
      do_import(choice.entry)
    end
  end)
end

-- Statusline helper
_G.sm_pomo_statusline = function()
  return require("sm").pomo_statusline()
end

-- Auto-refresh statusline every 30s
local pomo_timer = vim.loop.new_timer()
pomo_timer:start(30000, 30000, vim.schedule_wrap(function()
  vim.cmd("redrawstatus")
end))

-- Signcolumn + virtual text for TODO markers in normal buffers
local todo_augroup = vim.api.nvim_create_augroup("sm_todo_highlight", { clear = true })
local todo_patterns = {
  "TODO:",
  "FIXME:",
  "HACK:",
  "XXX:",
  "- %[ %]",
}

function SM_refresh_todo_highlights(buf)
  buf = buf or vim.api.nvim_get_current_buf()
  local ft = vim.bo[buf].filetype
  if ft == "sm" or ft == "qf" or ft == "markdown" then return end
  local ns = vim.api.nvim_create_namespace("sm_todo_virt")
  vim.api.nvim_buf_clear_namespace(buf, ns, 0, -1)
  vim.fn.sign_unplace("sm_todo", { buffer = buf })
  local lines = vim.api.nvim_buf_get_lines(buf, 0, -1)
  for lnum, line in ipairs(lines) do
    local marker
    for _, pat in ipairs(todo_patterns) do
      if line:find(pat) then
        marker = pat
        break
      end
    end
    if marker then
      local icon = marker:match("FIXME") and "FIX" or
                   marker:match("HACK") and "HCK" or
                   marker:match("XXX") and "XXX" or
                   "TODO"
      vim.api.nvim_buf_set_extmark(buf, ns, lnum - 1, -1, {
        sign_text = icon,
        sign_hl_group = "smTodoSign",
        priority = 200,
      })
    end
  end
end

-- Define highlight for TODO signs
vim.api.nvim_set_hl(0, "smTodoSign", { fg = "#e9b945", bold = true })

-- Autocmd to refresh highlights on open and save
vim.api.nvim_create_autocmd({ "BufRead", "BufWritePost" }, {
  group = todo_augroup,
  pattern = "*",
  callback = function(ctx)
    if vim.bo[ctx.buf].filetype == "sm" then return end
    SM_refresh_todo_highlights(ctx.buf)
  end,
})
-- On BufEnter for current buf after filetype set
vim.api.nvim_create_autocmd("FileType", {
  group = todo_augroup,
  pattern = "*",
  callback = function(ctx)
    if vim.bo[ctx.buf].filetype == "sm" then return end
    SM_refresh_todo_highlights(ctx.buf)
  end,
})

-- Commands
vim.api.nvim_create_user_command("SMTasks", function() SM_list() end, {})
vim.api.nvim_create_user_command("SMBacklog", function() SM_backlog() end, {})
vim.api.nvim_create_user_command("SMAdd", function(opts)
  SM_add(opts)
end, { nargs = "?", bang = true, complete = "file" })
vim.api.nvim_create_user_command("SMPomo", function(opts)
  SM_pomo(opts)
end, { nargs = 1, complete = "customlist,v:lua.sm_pomo_complete" })
vim.api.nvim_create_user_command("SMScan", function() SM_scan() end, {})
vim.api.nvim_create_user_command("SMImport", function() SM_import() end, {})
