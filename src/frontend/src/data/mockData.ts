// Mock data for NFT marketplace

export interface MockToken {
  id: string;
  name: string;
  image: string;
  price: string;
  volume: string;
  change: string;
  changePercent: string;
  isVerified?: boolean;
}

export interface MockCollection {
  id: string;
  name: string;
  floorPrice: string;
  volume: string;
  items: number;
  image: string;
  description?: string;
}

export interface MockNFT {
  id: string;
  name: string;
  image: string;
  price: string;
  collection: string;
}

// Top Movers data
export const topMovers: MockToken[] = [
  {
    id: "1",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
  {
    id: "2",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.3%",
    changePercent: "+325.3%",
    isVerified: true,
  },
  {
    id: "3",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
];

// Trending Tokens data
export const trendingTokens: MockToken[] = [
  {
    id: "4",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
  {
    id: "5",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
  {
    id: "6",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
];

// Featured Drop data
export const featuredDrop: MockCollection[] = [
  {
    id: "1",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "2",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "3",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
];

// Featured Collections data
export const featuredCollections: MockCollection[] = [
  {
    id: "4",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "5",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "6",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "7",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "8",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "9",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "10",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
  {
    id: "11",
    name: "OnChainBandits",
    floorPrice: "< 0.01 ETH",
    volume: "86 ETH",
    items: 1000,
    image: "/placeholder-collection.png",
  },
];

// Recently Updated data
export const recentlyUpdated: MockToken[] = [
  {
    id: "7",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
  {
    id: "8",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "-1.8%",
    changePercent: "-1.8%",
    isVerified: false,
  },
  {
    id: "9",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
  {
    id: "10",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
  {
    id: "11",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "-1.8%",
    changePercent: "-1.8%",
    isVerified: false,
  },
  {
    id: "12",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
  {
    id: "13",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "-1.8%",
    changePercent: "-1.8%",
    isVerified: false,
  },
  {
    id: "14",
    name: "Name",
    image: "/placeholder-nft.png",
    price: "14.89 ETH",
    volume: "86 ETH",
    change: "+325.4%",
    changePercent: "+325.4%",
    isVerified: true,
  },
];

// Collection detail NFTs
export const collectionNFTs: MockNFT[] = Array.from({ length: 20 }, (_, i) => ({
  id: `nft-${i + 1}`,
  name: "OnChainBandits",
  image: "/placeholder-nft.png",
  price: "< 0.01 ETH",
  collection: "OnChainBandits",
}));
