export const API_SERVER = (route: string) => `${process.env.API_URL || 'http://localhost:3005'}/api/${route}`
export const API_IPFS = (route: string) => `https://dweb.link/api/v0/${route}`
export const API_IPFS_GATEWAY = (hash: string) => `https://${hash}.ipfs.w3s.link`