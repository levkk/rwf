require 'yaml'

run do |env|
  puts env.to_yaml
  return [200, {}, ['Hello world']]
end
