.PHONY: init
init:
	@git config core.hooksPath .git-hooks || echo 'Not in a git repo'
