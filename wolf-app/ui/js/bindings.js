

El.bindings.el = function(el, tag, fallback) {
	if (!(El.cache[tag] || fallback)) return
	var tmp, tmp2
	, child = El(El.cache[tag] ? tag : fallback)
	if (tmp = el._scope) child._scope = tmp
	if (tmp = el.getAttribute("data-bind")) {
		tmp2 = child.getAttribute("data-bind")
		child.setAttribute("data-bind", tmp2 ? tmp + ";" + tmp2 : tmp)
	}
	if (tmp = el.className) El.cls(child, tmp)
	if (tmp = el.getAttribute("style")) child.setAttribute("style", tmp)
	el.parentNode.replaceChild(child, el)
	El.render(child, this)
	return child
}
El.bindings.el.once = true

