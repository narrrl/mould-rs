local M = {}

-- Helper function to get a set of files currently in a directory
local function get_dir_files(dir)
  local files = {}
  if vim.fn.isdirectory(dir) == 1 then
    for _, f in ipairs(vim.fn.readdir(dir)) do
      files[f] = true
    end
  end
  return files
end

local function open_floating_terminal(cmd, target_dir)
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

  -- Snapshot the directory contents BEFORE the command runs
  local files_before = get_dir_files(target_dir)

  vim.fn.termopen(cmd, {
    on_exit = function()
      -- Close the floating window and delete the terminal buffer
      vim.api.nvim_win_close(win, true)
      vim.api.nvim_buf_delete(buf, { force = true })
      
      -- Reload the original file if it exists
      if vim.fn.filereadable(original_file) == 1 then
        vim.schedule(function()
          vim.cmd("checktime " .. vim.fn.fnameescape(original_file))
        end)
      end

      -- Snapshot the directory AFTER the command finishes
      local files_after = get_dir_files(target_dir)
      
      -- Compare to find the newly created file
      for f, _ in pairs(files_after) do
        if not files_before[f] then
          local new_filepath = target_dir .. "/" .. f
          
          -- Open the new file (wrapped in schedule to ensure it runs safely after term closes)
          vim.schedule(function()
            vim.cmd("edit " .. vim.fn.fnameescape(new_filepath))
            -- Optional: Let the user know it worked
            vim.notify("mould.nvim: Opened " .. f, vim.log.levels.INFO)
          end)
          
          break -- Assuming only one file is generated; we can stop looking
        end
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

  -- Get the directory of the current file
  local target_dir = vim.fn.fnamemodify(filepath, ":p:h")
  local cmd = string.format("mould %s", vim.fn.shellescape(filepath))
  
  -- Pass the target directory to our terminal function
  open_floating_terminal(cmd, target_dir)
end

return M
