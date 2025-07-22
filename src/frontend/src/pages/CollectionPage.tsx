import { useState } from "react";
import { useParams } from "react-router-dom";
import { collectionNFTs } from "../data/mockData";

export function CollectionPage() {
  const { id } = useParams();
  const [sortBy, setSortBy] = useState("Price low to high");
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [showSortDropdown, setShowSortDropdown] = useState(false);

  const sortOptions = [
    "Price low to high",
    "Price high to low",
    "Most rare",
    "Least rare",
    "Recently listed",
    "Recently sold",
    "Recently created",
    "Recently transferred",
    "Highest sales",
    "Lowest sales",
    "Top offer",
  ];

  return (
    <div className="space-y-6 pb-24">
      {/* Collection Header */}
      <div className="flex items-start space-x-6">
        {/* Collection Avatar */}
        <div className="h-24 w-24 flex-shrink-0 rounded-lg bg-gray-700"></div>

        {/* Collection Info */}
        <div className="flex-1 space-y-4">
          <div className="flex items-center space-x-3">
            <h1 className="text-3xl font-bold">Featured Name</h1>
            <div className="flex items-center space-x-2">
              <button className="rounded-lg p-2 text-gray-400 transition-colors hover:bg-gray-800 hover:text-white">
                <svg
                  className="h-5 w-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
                  />
                </svg>
              </button>
              <button className="rounded-lg p-2 text-gray-400 transition-colors hover:bg-gray-800 hover:text-white">
                <svg
                  className="h-5 w-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z"
                  />
                </svg>
              </button>
            </div>
          </div>

          <div className="flex items-center space-x-6 text-sm">
            <span className="rounded bg-gray-800 px-2 py-1 text-white">
              JUL 2020
            </span>
            <span className="rounded bg-gray-800 px-2 py-1 text-white">
              ETHEREUM
            </span>
            <span className="rounded bg-gray-800 px-2 py-1 text-white">
              ART
            </span>
          </div>

          <div className="grid grid-cols-4 gap-4 text-center">
            <div>
              <p className="text-sm text-gray-400">최저가</p>
              <p className="text-xl font-bold">0.0225 ETF</p>
            </div>
            <div>
              <p className="text-sm text-gray-400">아이템</p>
              <p className="text-xl font-bold">9,999</p>
            </div>
            <div>
              <p className="text-sm text-gray-400">총 거래</p>
              <p className="text-xl font-bold">3.60 ETF</p>
            </div>
            <div>
              <p className="text-sm text-gray-400">등락률</p>
              <p className="text-xl font-bold text-green-500">1%</p>
            </div>
          </div>
        </div>

        {/* Collection Banner */}
        <div className="h-48 w-96 flex-shrink-0 rounded-lg bg-gray-700"></div>
      </div>

      {/* Navigation Tabs */}
      <div className="flex items-center space-x-6 border-b border-gray-800">
        <div className="flex items-center space-x-4">
          <button className="flex items-center space-x-2 border-b-2 border-purple-600 pb-4 font-medium text-purple-400">
            <svg
              className="h-4 w-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
            <span>1,234 Items</span>
          </button>

          <button className="pb-4 font-medium text-gray-400 transition-colors hover:text-white">
            Explore
          </button>

          <button className="pb-4 font-medium text-gray-400 transition-colors hover:text-white">
            Item
          </button>

          <button className="pb-4 font-medium text-gray-400 transition-colors hover:text-white">
            Offers
          </button>

          <button className="pb-4 font-medium text-gray-400 transition-colors hover:text-white">
            Holders
          </button>

          <button className="pb-4 font-medium text-gray-400 transition-colors hover:text-white">
            Traits
          </button>

          <button className="pb-4 font-medium text-gray-400 transition-colors hover:text-white">
            Activity
          </button>
        </div>
      </div>

      {/* Filter and Sort Bar */}
      <div className="flex items-center justify-between">
        {/* Sort Dropdown */}
        <div className="relative">
          <button
            onClick={() => setShowSortDropdown(!showSortDropdown)}
            className="flex items-center space-x-2 rounded-lg bg-gray-800 px-4 py-2 text-white transition-colors hover:bg-gray-700"
          >
            <span>Sort</span>
            <svg
              className="h-4 w-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 9l-7 7-7-7"
              />
            </svg>
          </button>

          {showSortDropdown && (
            <div className="absolute top-full left-0 z-10 mt-1 w-48 rounded-lg border border-gray-700 bg-gray-800 shadow-lg">
              {sortOptions.map((option) => (
                <button
                  key={option}
                  onClick={() => {
                    setSortBy(option);
                    setShowSortDropdown(false);
                  }}
                  className={`w-full px-4 py-2 text-left transition-colors first:rounded-t-lg last:rounded-b-lg hover:bg-gray-700 ${
                    sortBy === option ? "text-purple-400" : "text-white"
                  }`}
                >
                  {option}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* View Mode Toggle */}
        <div className="flex items-center space-x-2">
          <button
            onClick={() => setViewMode("grid")}
            className={`rounded-lg p-2 transition-colors ${
              viewMode === "grid"
                ? "bg-purple-600 text-white"
                : "bg-gray-800 text-gray-400 hover:text-white"
            }`}
          >
            <svg
              className="h-5 w-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"
              />
            </svg>
          </button>

          <button
            onClick={() => setViewMode("list")}
            className={`rounded-lg p-2 transition-colors ${
              viewMode === "list"
                ? "bg-purple-600 text-white"
                : "bg-gray-800 text-gray-400 hover:text-white"
            }`}
          >
            <svg
              className="h-5 w-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 6h16M4 10h16M4 14h16M4 18h16"
              />
            </svg>
          </button>
        </div>
      </div>

      {/* NFT Grid */}
      <div
        className={`grid gap-6 ${
          viewMode === "grid"
            ? "grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5"
            : "grid-cols-1"
        }`}
      >
        {collectionNFTs.map((nft) => (
          <div
            key={nft.id}
            className={`hover:bg-gray-750 group cursor-pointer rounded-lg bg-gray-800 transition-colors ${
              viewMode === "list" ? "flex items-center space-x-4 p-4" : "p-4"
            }`}
          >
            <div
              className={`flex-shrink-0 rounded-lg bg-gray-700 ${
                viewMode === "list" ? "h-16 w-16" : "mb-3 h-48 w-full"
              }`}
            ></div>

            <div className={viewMode === "list" ? "flex-1" : "space-y-2"}>
              <h3 className="font-medium transition-colors group-hover:text-purple-400">
                {nft.name}
              </h3>
              <div
                className={`text-sm ${viewMode === "list" ? "flex items-center space-x-4" : "space-y-1"}`}
              >
                <div
                  className={viewMode === "list" ? "" : "flex justify-between"}
                >
                  <span className="text-gray-400">FLOOR</span>
                  <span className="font-medium">{nft.price}</span>
                </div>
                <div
                  className={viewMode === "list" ? "" : "flex justify-between"}
                >
                  <span className="text-gray-400">24H VOLUME</span>
                  <span className="font-medium">86 ETH</span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Bottom Action Bar */}
      <div className="fixed right-0 bottom-0 left-0 z-40 border-t border-gray-800 bg-gray-900 p-4">
        <div className="ml-16 transition-all duration-300">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <button className="rounded-lg bg-purple-600 px-6 py-2 font-medium text-white transition-colors hover:bg-purple-700">
                Buy
              </button>
              <button className="rounded-lg bg-gray-800 px-6 py-2 font-medium text-white transition-colors hover:bg-gray-700">
                Sell
              </button>
            </div>

            <div className="flex items-center space-x-4">
              <div className="flex items-center space-x-2">
                <button className="rounded-lg bg-gray-800 p-2 text-gray-400 transition-colors hover:bg-gray-700 hover:text-white">
                  <svg
                    className="h-4 w-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M20 12H4"
                    />
                  </svg>
                </button>
                <span className="text-xl font-bold">0</span>
                <button className="rounded-lg bg-gray-800 p-2 text-gray-400 transition-colors hover:bg-gray-700 hover:text-white">
                  <svg
                    className="h-4 w-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 4v16m8-8H4"
                    />
                  </svg>
                </button>
              </div>

              <span className="text-xl font-bold">0</span>
              <span className="text-lg">ETF</span>

              <button className="rounded-lg bg-purple-600 px-6 py-2 font-medium text-white transition-colors hover:bg-purple-700">
                Buy floor
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
