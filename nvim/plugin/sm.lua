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
