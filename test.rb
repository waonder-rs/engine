require 'matrix'

def translation(x, y, z)
	Matrix[
		[1.0, 0.0, 0.0, x],
		[0.0, 1.0, 0.0, y],
		[0.0, 0.0, 1.0, z],
		[0.0, 0.0, 0.0, 1.0]
	]
end

def perspective(l, r, b, t, n, f)
	Matrix[
		[(2 * n) / (r - l), 0.0              , (r + l)/(r - l)    , 0.0                    ],
		[0.0              , (2 * n) / (t - b), (t + b) / (t - b)  , 0.0                    ],
		[0.0              , 0.0              , - (f + n) / (f - n), -1.0],
		[0.0              , 0.0              , - (2 * f * n) / (f - n), 0.0                    ]
	]
end

def fovx_perspective(fovx, aspect, n, f)
	r = n * Math::tan(fovx/2.0)
	t = r / aspect
	perspective(-r, r, -t, t, n, f)
end

def vec(x, y, z)
	Matrix[[x, y, z, 1]]
end

$model = translation(0.0, 0.0, 0.0)
$proj = perspective(-2.0, 2.0, -2.0, 2.0, 2.0, 6.0)

def project(x, y, z)
	p = vec(x, y, z) * $proj
	w = p[0, 3]
	[p[0, 0]/w, p[0, 1]/w, p[0, 2]/w]
end

puts project(1.0, 1.0, -3.0).inspect

# $proj = fovx_perspective(1.5707963267948966, 1.0, 2.0, 6.0)

puts project(1.0, 1.0, -3.0).inspect
