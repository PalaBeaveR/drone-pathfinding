module.exports = {
	syntax: require('postcss-less'),
	map: false,
	plugins: [require('postcss-nesting'), require('stylelint'), require('postcss-sorting')]
};
