if vim.g.loaded_mould == 1 then
  return
end
vim.g.loaded_mould = 1

vim.api.nvim_create_user_command("Mould", function()
  require("mould").open()
end, { desc = "Open mould for the current buffer" })
