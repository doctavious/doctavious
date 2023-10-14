module.exports = {
  plugins:[
    {
      resolve: `gatsby-plugin-output`,
      options: {
        publicPath: 'dist',
        rmPublicFolder: false
      }
    }
  ]
}
