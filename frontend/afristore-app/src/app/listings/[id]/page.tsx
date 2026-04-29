// ─────────────────────────────────────────────────────────────
// app/listings/[id]/page.tsx — Premium NFT listing detail page
// ─────────────────────────────────────────────────────────────

"use client";

import { useState, useEffect } from "react";
import { useParams, useRouter } from "next/navigation";
import Image from "next/image";
import {
    stroopsToXlm,
    Listing,
    Auction,
    getListing,
    getAuction,
    ListingStatus,
    AuctionStatus
} from "@/lib/contract";
import { fetchMetadata, cidToGatewayUrl, ArtworkMetadata } from "@/lib/ipfs";
import { useWalletContext } from "@/context/WalletContext";
import { useBuyArtwork, usePlaceBid } from "@/hooks/useMarketplace";
import { useListingOffers } from "@/hooks/useOffers";
import { useListingActivity } from "@/hooks/useUserActivity";
import { GuardButton } from "@/components/WalletGuard";
import {
    ArrowLeft,
    ExternalLink,
    ShoppingCart,
    User,
    Calendar,
    Tag,
    Hash,
    Clock,
    Gavel,
    History,
    ShieldCheck,
    CheckCircle2,
    AlertCircle,
    TrendingUp,
    Landmark,
    Share2,
} from "lucide-react";

export default function ListingDetailPage() {
    const { id } = useParams<{ id: string }>();
    const router = useRouter();
    const { publicKey } = useWalletContext();

    // State
    const [listing, setListing] = useState<Listing | null>(null);
    const [auction, setAuction] = useState<Auction | null>(null);
    const [metadata, setMetadata] = useState<ArtworkMetadata | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState<'details' | 'history' | 'offers'>('details');

    // Hooks
    const { buy, isBuying, error: buyError } = useBuyArtwork(publicKey);
    const { bid, isBidding, error: bidError } = usePlaceBid(publicKey);
    const { offers, isLoading: isLoadingOffers, refresh: refreshOffers } = useListingOffers(id ? Number(id) : null);
    const { activities, isLoading: isLoadingActivity } = useListingActivity(id ? Number(id) : null);

    useEffect(() => {
        const loadData = async () => {
            if (!id) return;
            setIsLoading(true);
            setError(null);
            try {
                // Try fetching as listing first
                let l: Listing | null = null;
                let a: Auction | null = null;

                try {
                    l = await getListing(Number(id));
                    setListing(l);
                } catch (e) {
                    // Might be an auction only or not found
                    console.log("Not found as listing, checking auction...");
                }

                try {
                    a = await getAuction(Number(id));
                    setAuction(a);
                } catch (e) {
                    // Might be a listing only
                    console.log("Not found as auction.");
                }

                if (!l && !a) {
                    throw new Error("Artwork not found on-chain");
                }

                const cid = l?.metadata_cid || a?.metadata_cid;
                if (cid) {
                    const m = await fetchMetadata(cid);
                    setMetadata(m);
                }
            } catch (err: any) {
                console.error("Error loading data:", err);
                setError(err.message || "Failed to load artwork details");
            } finally {
                setIsLoading(false);
            }
        };
        loadData();
    }, [id]);

    const handleBuy = async () => {
        if (!listing) return;
        const success = await buy(listing.listing_id);
        if (success) {
            // Reload
            const updated = await getListing(listing.listing_id);
            setListing(updated);
        }
    };

    const [bidAmount, setBidAmount] = useState("");
    const handleBid = async () => {
        if (!auction || !bidAmount) return;
        const success = await bid(auction.auction_id, Number(bidAmount));
        if (success) {
            const updated = await getAuction(auction.auction_id);
            setAuction(updated);
            setBidAmount("");
        }
    };

export async function generateMetadata({ params }: ListingPageProps): Promise<Metadata> {
  const { id } = await params
  const baseUrl = config.baseUrl
  const listingId = parseInt(id)
  
  if (isNaN(listingId)) {
    return {
      title: 'Artwork Not Found | Afristore',
      description: 'The artwork you are looking for does not exist or has been removed.',
    }
  }

  try {
    // Try to fetch listing first
    let listing = null
    let auction = null
    let metadata = null

    try {
      listing = await getListing(listingId)
    } catch (e) {
      // Try auction if listing fails
      try {
        auction = await getAuction(listingId)
      } catch (e) {
        // Neither found
      }
    }

    if (!listing && !auction) {
      return {
        title: 'Artwork Not Found | Afristore',
        description: 'The artwork you are looking for does not exist or has been removed.',
      }
    }

    // Fetch metadata
    const cid = listing?.metadata_cid || auction?.metadata_cid
    if (cid) {
      metadata = await fetchMetadata(cid)
    }

    const artist = listing?.artist || auction?.creator
    const price = listing ? stroopsToXlm(listing.price) : auction ? stroopsToXlm(auction.highest_bid || auction.reserve_price) : '0'
    const status = listing?.status || auction?.status

    const title = metadata?.title || `Artwork #${id}`
    const description = metadata?.description || `Discover this unique African digital artwork on the Stellar blockchain. ${listing ? 'Fixed price' : 'Auction'} listing.`
    const imageUrl = metadata?.image ? cidToGatewayUrl(metadata.image) : `${baseUrl}/api/og/listing/${id}`

    return {
      title: `${title} | ${artist?.slice(0, 6)}…${artist?.slice(-4)} | Afristore`,
      description,
      openGraph: {
        title,
        description,
        type: 'article',
        url: `${baseUrl}/listings/${id}`,
        images: [
          {
            url: imageUrl,
            width: 1200,
            height: 630,
            alt: `${title} by ${artist?.slice(0, 6)}…${artist?.slice(-4)}`,
          },
        ],
        siteName: 'Afristore - African Art Marketplace',
      },
      twitter: {
        card: 'summary_large_image',
        title,
        description,
        images: [imageUrl],
        creator: '@afristore',
        site: '@afristore',
      },
      alternates: {
        canonical: `${baseUrl}/listings/${id}`,
      },
      keywords: [
        'African art',
        'NFT',
        'Stellar blockchain',
        'Digital art',
        metadata?.category || 'Art',
        metadata?.artist || 'African artist',
        title,
        'Crypto art',
        'Blockchain art',
        listing ? 'Buy art' : 'Bid on art',
      ],
      authors: metadata?.artist ? [{ name: metadata.artist }] : undefined,
      category: 'art',
    }
  } catch (error) {
    console.error('Failed to generate metadata for listing:', error)
    
    return {
      title: `Artwork #${id} | Afristore`,
      description: 'Discover this unique African digital artwork on the Stellar blockchain.',
    }
  }
}

export default async function ListingPage({ params }: ListingPageProps) {
  const { id } = await params
  return <ListingClient id={id} />
}
