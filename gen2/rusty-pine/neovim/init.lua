local lsp = require 'lspconfig'
local lsp_configs = require 'lspconfig.configs'
local util = require 'lspconfig.util'

-- To get this to work you must load this file in Neovim
--  luafile <path to this file>/init.lua

vim.lsp.set_log_level("debug")

vim.filetype.add({
  extension = {
    pine = "pine",
  }
})

lsp_configs.rusty_pine = {
    default_config = {
        -- you may need to edit the path
        cmd = { "rusty-pine" },
        single_file_support = true,
        root_dir = function(fname)
            return util.path.dirname(fname)
        end,
        filetypes = { "pine" },
        codeAction = {
            disableRuleComment = {
              enable = true,
              location = 'separateLine',
            },
            showDocumentation = {
              enable = true,
            },
        }
    }
}

-- Use LspAttach autocommand to only map the following keys
-- after the language server attaches to the current buffer
vim.api.nvim_create_autocmd('LspAttach', {
  group = vim.api.nvim_create_augroup('UserLspConfig', {}),
  callback = function(ev)
    vim.notify(('[lspconfig] LspAttach'), vim.log.levels.WARN)
    -- Enable completion triggered by <c-x><c-o>
    vim.bo[ev.buf].omnifunc = 'v:lua.vim.lsp.omnifunc'

    -- Buffer local mappings.
    -- See `:help vim.lsp.*` for documentation on any of the below functions
    local opts = { buffer = ev.buf }
    vim.keymap.set({ 'n', 'v' }, '<space>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', '<space>f', function()
      vim.lsp.buf.format { async = true }
    end, opts)
  end,
})

lsp.rusty_pine.setup{
}
