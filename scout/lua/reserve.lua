local resource = KEYS[1]
local claim = ARGV[1]
local current = redis.call('GET', resource)
if not current then
    -- Freshly reserved
    redis.call('SET', resource, claim)
    return 1
elseif current == claim then
    -- Already reserved
    return 0
end
return {err='BadClaim'}
