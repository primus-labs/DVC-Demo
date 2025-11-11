const fs = require('fs');

/**
 * Save data to file
 */
function saveToFile(filepath, data) {
  fs.writeFileSync(filepath, data);
}

/**
 * Sleep helper (ms)
 */
async function sleepMs(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

module.exports = { saveToFile, sleepMs };
