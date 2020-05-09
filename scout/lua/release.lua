local resource = KEYS[1]
local claim = ARGV[1]
local current = redis.call('GET', resource)
if not current then
    return {err='NoClaim'}
elseif current ~= claim then
    return {err='BadClaim'}
end
redis.call('DEL', resource)
