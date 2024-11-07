#!/usr/bin/env ruby
# Source: https://silverhammermba.github.io/emberb/embed/
require 'shellwords'

# location of ruby.h
hdrdir = Shellwords.escape RbConfig::CONFIG["rubyhdrdir"]
# location of ruby/config.h
archhdrdir = Shellwords.escape RbConfig::CONFIG["rubyarchhdrdir"]
# location of libruby
libdir = Shellwords.escape RbConfig::CONFIG["libdir"]

# args for GCC
puts "-I#{hdrdir} -I#{archhdrdir} -L#{libdir} -Wl,-rpath,#{libdir}"
