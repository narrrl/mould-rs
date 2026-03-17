local M = {}

local function open_floating_terminal(cmd)
  local buf = vim.api.nvim_create_buf(false, true)
  local width = math.floor(vim.o.columns * 0.9)
  local height = math.floor(vim.o.lines * 0.9)
  local col = math.floor((vim.o.columns - width) / 2)
  local row = math.floor((vim.o.lines - height) / 2)

  local win_config = {
    relative = "editor",
    width = width,
    height = height,
    col = col,
    row = row,
    style = "minimal",
    border = "rounded",
  }

  local win = vim.api.nvim_open_win(buf, true, win_config)
  
  -- Record the original buffer to reload it later
  local original_buf = vim.api.nvim_get_current_buf()
  local original_file = vim.api.nvim_buf_get_name(original_buf)

  vim.fn.termopen(cmd, {
    on_exit = function()
      vim.api.nvim_win_close(win, true)
      vim.api.nvim_buf_delete(buf, { force = true })
      
      -- Reload the original file if it exists
      if vim.fn.filereadable(original_file) == 1 then
        vim.schedule(function()
          vim.cmd("checktime " .. vim.fn.fnameescape(original_file))
        end)
      end
    end,
  })

  vim.cmd("startinsert")
end

function M.open()
  local filepath = vim.api.nvim_buf_get_name(0)
  if filepath == "" then
    vim.notify("mould.nvim: Cannot open mould for an unnamed buffer.", vim.log.levels.ERROR)
    return
  end

  local cmd = string.format("mould %s", vim.fn.shellescape(filepath))
  open_floating_terminal(cmd)
end

return M
