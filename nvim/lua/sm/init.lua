local M = {}

local sm_bin = vim.fn.getenv("SM_BIN")
if sm_bin == vim.NIL then
  -- procura no projeto atual
  local cwd = vim.fn.getcwd()
  local candidates = {
    cwd .. "/target/debug/sm.exe",
    cwd .. "/target/debug/sm",
    cwd .. "/sm.exe",
    cwd .. "/sm",
    "sm",
  }
  for _, p in ipairs(candidates) do
    if vim.fn.executable(p) == 1 then
      sm_bin = p
      break
    end
  end
  if sm_bin == vim.NIL then
    sm_bin = "sm"
  end
end

function M.run(args, cb)
  local full_cmd = sm_bin .. " " .. args
  vim.system({ sm_bin }, { args = vim.split(args, " ") }, function(out)
    if cb then
      cb(out)
    end
  end)
end

function M.run_sync(args)
  local cmd = sm_bin .. " " .. args
  local handle = io.popen(cmd, "r")
  if not handle then return "", 1 end
  local result = handle:read("*a")
  local ok = handle:close()
  return result, ok
end

function M.notify(msg, level)
  level = level or vim.log.levels.INFO
  local ok, noice = pcall(require, "noice")
  if ok and noice.notify then
    noice.notify(msg, level)
  else
    vim.notify(msg, level)
  end
end

function M.list(daily)
  local flag = daily and "" or " --all"
  local res = M.run_sync("list" .. flag)
  return res
end

function M.backlog()
  return M.run_sync("backlog")
end

function M.add(task, tags, today)
  local tag_part = tags and #tags > 0 and " --tags " .. table.concat(tags, ",") or ""
  local today_flag = today and " --today" or ""
  local res = M.run_sync("add --task \"" .. task .. "\"" .. tag_part .. today_flag)
  return res
end

function M.pull(id)
  if id then
    return M.run_sync("pull --id " .. id)
  end
  return M.run_sync("pull --all")
end

function M.toggle(id)
  return M.run_sync("toggle --id " .. id)
end

function M.remove(id)
  return M.run_sync("remove --id " .. id)
end

function M.cancel(id)
  return M.run_sync("cancel --id " .. id)
end

function M.pomo_start()
  return M.run_sync("pomo start")
end

function M.pomo_stop()
  return M.run_sync("pomo stop")
end

function M.pomo_status()
  return M.run_sync("pomo status")
end

function M.parse_line(line)
  local bullet, id, desc = line:match("^(%S+) (%d+) %- (.+)$")
  if not bullet then return nil end
  local tags = {}
  local cleaned = desc
  for tag in desc:gmatch("#(%S+)") do
    table.insert(tags, tag)
    cleaned = cleaned:gsub("#" .. tag, "")
  end
  cleaned = cleaned:gsub("%[%w+%]", ""):gsub("%s+", " "):gsub("^%s+", ""):gsub("%s+$", "")
  return { bullet = bullet, id = tonumber(id), description = cleaned, raw = desc, tags = tags }
end

function M.pomo_statusline()
  local res = M.run_sync("pomo status")
  local cleaned = vim.trim(res or "")
  if cleaned == "" or cleaned:match("No active") then
    return ""
  end
  local time = cleaned:match("(%d+:%d+)")
  if not time then return "" end
  if cleaned:match("Focus") then
    return "▸ " .. time
  end
  if cleaned:match("Break") then
    return "◷ " .. time
  end
  return time
end

return M
