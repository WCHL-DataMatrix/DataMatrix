import {
  topMovers,
  trendingTokens,
  featuredDrop,
  featuredCollections,
  recentlyUpdated,
} from "../data/mockData";
import { Link } from "react-router-dom";

export function HomePage() {
  return (
    <div className="space-y-8">
      {/* Top Section - Large Hero Area + Sidebar with Top Mover & Trending Token */}
      <div className="flex gap-8">
        {/* Left Side - Hero Area + Featured Drop */}
        <div className="flex-1 space-y-8">
          {/* Large Hero/Featured Area - Smaller height */}
          <div className="h-64 w-full rounded-lg bg-gray-800"></div>

          {/* Featured Drop */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <svg
                  className="h-6 w-6 text-white"
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
                <h2 className="text-xl font-bold">Featured Drop</h2>
              </div>
              <Link
                to="/collections"
                className="text-sm font-medium text-purple-400 hover:text-purple-300"
              >
                View category
              </Link>
            </div>

            <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
              {featuredDrop.map((collection) => (
                <Link
                  key={collection.id}
                  to={`/collection/${collection.id}`}
                  className="hover:bg-gray-750 group rounded-lg bg-gray-800 p-6 transition-colors"
                >
                  <div className="mb-4 h-48 w-full rounded-lg bg-gray-700"></div>
                  <div className="space-y-2">
                    <h3 className="text-lg font-bold transition-colors group-hover:text-purple-400">
                      {collection.name}
                    </h3>
                    <div className="flex justify-between text-sm">
                      <div>
                        <p className="text-gray-400">FLOOR</p>
                        <p className="font-medium">{collection.floorPrice}</p>
                      </div>
                      <div className="text-right">
                        <p className="text-gray-400">24H VOLUME</p>
                        <p className="font-medium">{collection.volume}</p>
                      </div>
                    </div>
                  </div>
                </Link>
              ))}
            </div>
          </div>
        </div>

        {/* Right Sidebar - Fixed width, matches total height of left side */}
        <div className="w-80 space-y-6">
          {/* Top Mover Today */}
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <svg
                className="h-5 w-5 text-yellow-500"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M12.395 2.553a1 1 0 00-1.45-.385c-.345.23-.614.558-.822.88-.214.33-.403.713-.57 1.116-.334.804-.614 1.768-.84 2.734a31.365 31.365 0 00-.613 3.58 2.64 2.64 0 01-.945-1.067c-.328-.68-.398-1.534-.398-2.654A1 1 0 005.05 6.05 6.981 6.981 0 003 11a7 7 0 1011.95-4.95c-.592-.591-.98-.985-1.348-1.467-.363-.476-.724-1.063-1.207-2.03zM12.12 15.12A3 3 0 017 13s.879.5 2.5.5c0-1 .5-4 1.25-4.5.5 1 .786 1.293 1.371 1.879A2.99 2.99 0 0113 13a2.99 2.99 0 01-.879 2.121z"
                  clipRule="evenodd"
                />
              </svg>
              <h2 className="text-lg font-bold">Top Mover Today</h2>
            </div>

            <div className="space-y-3">
              {topMovers.map((token, index) => (
                <div
                  key={token.id}
                  className="hover:bg-gray-750 flex items-center space-x-3 rounded-lg bg-gray-800 p-3 transition-colors"
                >
                  <span className="w-6 text-xl font-bold text-gray-400">
                    {index + 1}
                  </span>
                  <div className="h-10 w-10 flex-shrink-0 rounded-lg bg-gray-700"></div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center space-x-1">
                      <span className="text-sm font-medium">{token.name}</span>
                      {token.isVerified && (
                        <svg
                          className="h-3 w-3 text-blue-500"
                          fill="currentColor"
                          viewBox="0 0 20 20"
                        >
                          <path
                            fillRule="evenodd"
                            d="M6.267 3.455a3.066 3.066 0 001.745-.723 3.066 3.066 0 013.976 0 3.066 3.066 0 001.745.723 3.066 3.066 0 012.812 2.812c.051.643.304 1.254.723 1.745a3.066 3.066 0 010 3.976 3.066 3.066 0 00-.723 1.745 3.066 3.066 0 01-2.812 2.812 3.066 3.066 0 00-1.745.723 3.066 3.066 0 01-3.976 0 3.066 3.066 0 00-1.745-.723 3.066 3.066 0 01-2.812-2.812 3.066 3.066 0 00-.723-1.745 3.066 3.066 0 010-3.976 3.066 3.066 0 00.723-1.745 3.066 3.066 0 012.812-2.812zm7.44 5.252a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clipRule="evenodd"
                          />
                        </svg>
                      )}
                    </div>
                    <p className="text-xs text-gray-400">&lt; {token.price}</p>
                  </div>
                  <div className="text-right">
                    <p className="text-xs font-bold text-green-500">
                      {token.changePercent}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Trending Token */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <svg
                  className="h-5 w-5 text-purple-500"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"
                  />
                </svg>
                <h2 className="text-lg font-bold">Trending Token</h2>
              </div>
              <Link
                to="/token"
                className="text-xs font-medium text-purple-400 hover:text-purple-300"
              >
                View category
              </Link>
            </div>

            <div className="space-y-3">
              {trendingTokens.map((token) => (
                <div
                  key={token.id}
                  className="hover:bg-gray-750 flex items-center space-x-3 rounded-lg bg-gray-800 p-3 transition-colors"
                >
                  <div className="h-10 w-10 flex-shrink-0 rounded-lg bg-gray-700"></div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center space-x-1">
                      <span className="text-sm font-medium">{token.name}</span>
                      {token.isVerified && (
                        <svg
                          className="h-3 w-3 text-blue-500"
                          fill="currentColor"
                          viewBox="0 0 20 20"
                        >
                          <path
                            fillRule="evenodd"
                            d="M6.267 3.455a3.066 3.066 0 001.745-.723 3.066 3.066 0 013.976 0 3.066 3.066 0 001.745.723 3.066 3.066 0 012.812 2.812c.051.643.304 1.254.723 1.745a3.066 3.066 0 010 3.976 3.066 3.066 0 00-.723 1.745 3.066 3.066 0 01-2.812 2.812 3.066 3.066 0 00-1.745.723 3.066 3.066 0 01-3.976 0 3.066 3.066 0 00-1.745-.723 3.066 3.066 0 01-2.812-2.812 3.066 3.066 0 00-.723-1.745 3.066 3.066 0 010-3.976 3.066 3.066 0 00.723-1.745 3.066 3.066 0 012.812-2.812zm7.44 5.252a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clipRule="evenodd"
                          />
                        </svg>
                      )}
                    </div>
                    <p className="text-xs text-gray-400">&lt; {token.price}</p>
                  </div>
                  <div className="text-right">
                    <p className="text-xs font-bold text-green-500">
                      {token.changePercent}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Featured Drop */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <svg
              className="h-6 w-6 text-white"
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
            <h2 className="text-xl font-bold">Featured Drop</h2>
          </div>
          <Link
            to="/collections"
            className="text-sm font-medium text-purple-400 hover:text-purple-300"
          >
            View category
          </Link>
        </div>

        <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
          {featuredDrop.map((collection) => (
            <Link
              key={collection.id}
              to={`/collection/${collection.id}`}
              className="hover:bg-gray-750 group rounded-lg bg-gray-800 p-6 transition-colors"
            >
              <div className="mb-4 h-48 w-full rounded-lg bg-gray-700"></div>
              <div className="space-y-2">
                <h3 className="text-lg font-bold transition-colors group-hover:text-purple-400">
                  {collection.name}
                </h3>
                <div className="flex justify-between text-sm">
                  <div>
                    <p className="text-gray-400">FLOOR</p>
                    <p className="font-medium">{collection.floorPrice}</p>
                  </div>
                  <div className="text-right">
                    <p className="text-gray-400">24H VOLUME</p>
                    <p className="font-medium">{collection.volume}</p>
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {/* Category Tabs */}
      <div className="space-y-4">
        <div className="flex items-center space-x-4">
          <button className="rounded-lg bg-purple-600 px-4 py-2 font-medium text-white">
            Category
          </button>
          <button className="rounded-lg bg-purple-600 px-4 py-2 font-medium text-white">
            Category 1
          </button>
          <button className="rounded-lg bg-gray-700 px-4 py-2 font-medium text-gray-300 transition-colors hover:bg-gray-600">
            Category 2
          </button>
          <button className="rounded-lg bg-gray-700 px-4 py-2 font-medium text-gray-300 transition-colors hover:bg-gray-600">
            Category 3
          </button>
          <div className="flex-1"></div>
          <Link
            to="/collections"
            className="text-sm font-medium text-purple-400 hover:text-purple-300"
          >
            See all
          </Link>
          <div className="flex items-center space-x-2">
            <button className="rounded-lg bg-gray-700 p-2 text-gray-400 transition-colors hover:bg-gray-600 hover:text-white">
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
                  d="M15 19l-7-7 7-7"
                />
              </svg>
            </button>
            <button className="rounded-lg bg-gray-700 p-2 text-gray-400 transition-colors hover:bg-gray-600 hover:text-white">
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
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </button>
            <button className="rounded-lg p-2 text-gray-400 transition-colors hover:text-white">
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
                  d="M5 12h.01M12 12h.01M19 12h.01M6 12a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0z"
                />
              </svg>
            </button>
            <button className="rounded-lg p-2 text-gray-400 transition-colors hover:text-white">
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
                  d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                />
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Featured Collections */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-bold">Featured Collections</h2>
          <Link
            to="/collections"
            className="text-sm font-medium text-purple-400 hover:text-purple-300"
          >
            View category
          </Link>
        </div>

        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-4">
          {featuredCollections.map((collection) => (
            <Link
              key={collection.id}
              to={`/collection/${collection.id}`}
              className="hover:bg-gray-750 group rounded-lg bg-gray-800 p-4 transition-colors"
            >
              <div className="mb-3 h-32 w-full rounded-lg bg-gray-700"></div>
              <div className="space-y-2">
                <h3 className="font-medium transition-colors group-hover:text-purple-400">
                  {collection.name}
                </h3>
                <div className="flex justify-between text-sm">
                  <div>
                    <p className="text-gray-400">FLOOR</p>
                    <p className="font-medium">{collection.floorPrice}</p>
                  </div>
                  <div className="text-right">
                    <p className="text-gray-400">24H VOLUME</p>
                    <p className="font-medium">{collection.volume}</p>
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {/* Recently Updated */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-xl font-bold">Recently Updated</h2>
            <p className="text-sm text-gray-400">
              This week's curated collections
            </p>
          </div>
          <Link
            to="/collections"
            className="text-sm font-medium text-purple-400 hover:text-purple-300"
          >
            View category
          </Link>
        </div>

        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-4">
          {recentlyUpdated.map((token) => (
            <div
              key={token.id}
              className="hover:bg-gray-750 group rounded-lg bg-gray-800 p-4 transition-colors"
            >
              <div className="mb-3 h-32 w-full rounded-lg bg-gray-700"></div>
              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <span className="font-medium transition-colors group-hover:text-purple-400">
                    {token.name}
                  </span>
                  {token.isVerified && (
                    <svg
                      className="h-4 w-4 text-blue-500"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fillRule="evenodd"
                        d="M6.267 3.455a3.066 3.066 0 001.745-.723 3.066 3.066 0 013.976 0 3.066 3.066 0 001.745.723 3.066 3.066 0 012.812 2.812c.051.643.304 1.254.723 1.745a3.066 3.066 0 010 3.976 3.066 3.066 0 00-.723 1.745 3.066 3.066 0 01-2.812 2.812 3.066 3.066 0 00-1.745.723 3.066 3.066 0 01-3.976 0 3.066 3.066 0 00-1.745-.723 3.066 3.066 0 01-2.812-2.812 3.066 3.066 0 00-.723-1.745 3.066 3.066 0 010-3.976 3.066 3.066 0 00.723-1.745 3.066 3.066 0 012.812-2.812zm7.44 5.252a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                        clipRule="evenodd"
                      />
                    </svg>
                  )}
                </div>
                <div className="flex justify-between text-sm">
                  <p className="text-gray-400">&lt; {token.price}</p>
                  <p
                    className={`font-medium ${
                      token.change.startsWith("+")
                        ? "text-green-500"
                        : "text-red-500"
                    }`}
                  >
                    {token.changePercent}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
