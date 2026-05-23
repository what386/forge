local render_native = forge.__render_native
local render_to_native = forge.__render_to_native
local render_dir_native = forge.__render_dir_native
local render_dir_to_native = forge.__render_dir_to_native

local function capture_scope(caller_func)
    local scope = {}
    local index = 1
    while true do
        local name, value = debug.getlocal(3, index)
        if name == nil then break end
        if name ~= "(*temporary)" and string.sub(name, 1, 1) ~= "(" then
            scope[name] = value
        end
        index = index + 1
    end

    if caller_func then
        index = 1
        while true do
            local name, value = debug.getupvalue(caller_func, index)
            if name == nil then break end
            if name ~= "_ENV" and scope[name] == nil then
                scope[name] = value
            end
            index = index + 1
        end
    end
    return scope
end

forge.render = function(src)
    local caller = debug.getinfo(2, "f")
    return render_native(src, capture_scope(caller and caller.func))
end

forge.render_to = function(src, dst)
    local caller = debug.getinfo(2, "f")
    return render_to_native(src, dst, capture_scope(caller and caller.func))
end

forge.render_dir = function(src)
    local caller = debug.getinfo(2, "f")
    return render_dir_native(src, capture_scope(caller and caller.func))
end

forge.render_dir_to = function(src, dst)
    local caller = debug.getinfo(2, "f")
    return render_dir_to_native(src, dst, capture_scope(caller and caller.func))
end

forge.__render_native = nil
forge.__render_to_native = nil
forge.__render_dir_native = nil
forge.__render_dir_to_native = nil
