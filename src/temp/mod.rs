pub type Temperature = u16; // This needs to be wrapped correct to be a 12 bit vector but BitVec isn't in stable yet
                            // More importantly this currently means you can't set more then 16
                            // nodes because of the sizing
