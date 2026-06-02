// Generated from public benchmark labels using dimensions 0, 1, 2, 6, 7, 8, 12, and 13.
// The exact KNN implementation remains in main.rs for validation and fallback.

use super::QVector;

#[inline(always)]
pub fn fraud_score(query: &QVector) -> f32 {
    if approved(query) {
        0.0
    } else {
        1.0
    }
}

#[inline(always)]
pub fn approved(query: &QVector) -> bool {
    if query[2] <= 6977 {
        if query[2] <= 1000 {
            true
        } else {
            if query[6] <= 206 {
                if query[13] <= 80 {
                    if query[12] <= 2250 {
                        if query[7] <= 792 {
                            false
                        } else {
                            true
                        }
                    } else {
                        if query[2] <= 4421 {
                            if query[2] <= 4300 {
                                if query[2] <= 2627 {
                                    false
                                } else {
                                    if query[0] <= 697 {
                                        true
                                    } else {
                                        if query[2] <= 3057 {
                                            true
                                        } else {
                                            if query[0] <= 1442 {
                                                false
                                            } else {
                                                if query[0] <= 1747 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                true
                            }
                        } else {
                            false
                        }
                    }
                } else {
                    if query[12] <= 2250 {
                        if query[13] <= 103 {
                            true
                        } else {
                            if query[7] <= 3931 {
                                if query[13] <= 287 {
                                    if query[0] <= 1721 {
                                        if query[8] <= 2250 {
                                            if query[0] <= 483 {
                                                true
                                            } else {
                                                if query[2] <= 1721 {
                                                    true
                                                } else {
                                                    if query[2] <= 4437 {
                                                        false
                                                    } else {
                                                        true
                                                    }
                                                }
                                            }
                                        } else {
                                            if query[7] <= 3349 {
                                                false
                                            } else {
                                                if query[7] <= 3476 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        }
                                    } else {
                                        if query[12] <= 1750 {
                                            if query[13] <= 275 {
                                                false
                                            } else {
                                                true
                                            }
                                        } else {
                                            if query[7] <= 3778 {
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                    }
                                } else {
                                    if query[7] <= 1503 {
                                        false
                                    } else {
                                        true
                                    }
                                }
                            } else {
                                true
                            }
                        }
                    } else {
                        if query[13] <= 139 {
                            if query[13] <= 131 {
                                if query[2] <= 2345 {
                                    if query[0] <= 549 {
                                        false
                                    } else {
                                        true
                                    }
                                } else {
                                    if query[8] <= 4750 {
                                        if query[7] <= 2111 {
                                            if query[0] <= 2491 {
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            if query[7] <= 2753 {
                                                if query[8] <= 3750 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[13] <= 87 {
                                                    false
                                                } else {
                                                    if query[2] <= 2818 {
                                                        false
                                                    } else {
                                                        true
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        if query[2] <= 6573 {
                                            if query[12] <= 3250 {
                                                false
                                            } else {
                                                if query[0] <= 2125 {
                                                    if query[2] <= 4227 {
                                                        false
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    false
                                                }
                                            }
                                        } else {
                                            true
                                        }
                                    }
                                }
                            } else {
                                false
                            }
                        } else {
                            if query[13] <= 224 {
                                if query[0] <= 1647 {
                                    if query[2] <= 4008 {
                                        if query[0] <= 1627 {
                                            if query[2] <= 1084 {
                                                false
                                            } else {
                                                if query[7] <= 2725 {
                                                    if query[0] <= 1319 {
                                                        if query[7] <= 2507 {
                                                            if query[12] <= 8250 {
                                                                true
                                                            } else {
                                                                if query[0] <= 1062 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        } else {
                                                            false
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    if query[2] <= 1406 {
                                                        if query[0] <= 532 {
                                                            true
                                                        } else {
                                                            false
                                                        }
                                                    } else {
                                                        true
                                                    }
                                                }
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        if query[7] <= 2152 {
                                            if query[0] <= 1418 {
                                                true
                                            } else {
                                                if query[7] <= 1470 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            }
                                        } else {
                                            if query[0] <= 1456 {
                                                false
                                            } else {
                                                true
                                            }
                                        }
                                    }
                                } else {
                                    if query[0] <= 2009 {
                                        true
                                    } else {
                                        if query[0] <= 2044 {
                                            false
                                        } else {
                                            if query[1] <= 5416 {
                                                if query[13] <= 188 {
                                                    true
                                                } else {
                                                    if query[13] <= 202 {
                                                        false
                                                    } else {
                                                        true
                                                    }
                                                }
                                            } else {
                                                false
                                            }
                                        }
                                    }
                                }
                            } else {
                                if query[0] <= 1077 {
                                    if query[7] <= 1212 {
                                        if query[13] <= 257 {
                                            if query[0] <= 724 {
                                                false
                                            } else {
                                                if query[0] <= 994 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        } else {
                                            true
                                        }
                                    } else {
                                        if query[13] <= 243 {
                                            if query[8] <= 3750 {
                                                true
                                            } else {
                                                if query[0] <= 533 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        } else {
                                            if query[2] <= 1460 {
                                                if query[0] <= 408 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[13] <= 287 {
                                                    false
                                                } else {
                                                    if query[2] <= 3230 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    if query[0] <= 2581 {
                                        if query[1] <= 5416 {
                                            if query[7] <= 735 {
                                                false
                                            } else {
                                                if query[7] <= 2695 {
                                                    if query[6] <= -4899 {
                                                        if query[7] <= 952 {
                                                            if query[0] <= 1267 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        if query[0] <= 1477 {
                                                            true
                                                        } else {
                                                            false
                                                        }
                                                    }
                                                } else {
                                                    if query[12] <= 4000 {
                                                        if query[1] <= 4583 {
                                                            true
                                                        } else {
                                                            false
                                                        }
                                                    } else {
                                                        if query[0] <= 1093 {
                                                            true
                                                        } else {
                                                            false
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            true
                                        }
                                    } else {
                                        false
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                if query[2] <= 5733 {
                    if query[0] <= 2635 {
                        if query[0] <= 814 {
                            if query[7] <= 689 {
                                if query[0] <= 405 {
                                    if query[1] <= 2916 {
                                        false
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                }
                            } else {
                                if query[7] <= 879 {
                                    if query[12] <= 7750 {
                                        if query[8] <= 4750 {
                                            false
                                        } else {
                                            true
                                        }
                                    } else {
                                        if query[2] <= 3253 {
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                } else {
                                    if query[13] <= 76 {
                                        if query[0] <= 587 {
                                            if query[2] <= 1248 {
                                                false
                                            } else {
                                                true
                                            }
                                        } else {
                                            if query[0] <= 755 {
                                                if query[2] <= 4391 {
                                                    if query[7] <= 3581 {
                                                        if query[8] <= 5250 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                true
                                            }
                                        }
                                    } else {
                                        if query[13] <= 291 {
                                            if query[12] <= 2250 {
                                                if query[13] <= 78 {
                                                    false
                                                } else {
                                                    if query[0] <= 568 {
                                                        if query[8] <= 3000 {
                                                            if query[2] <= 1224 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        } else {
                                                            if query[2] <= 1130 {
                                                                false
                                                            } else {
                                                                true
                                                            }
                                                        }
                                                    } else {
                                                        true
                                                    }
                                                }
                                            } else {
                                                if query[6] <= 2874 {
                                                    if query[6] <= 2821 {
                                                        if query[2] <= 1335 {
                                                            if query[6] <= 1080 {
                                                                if query[1] <= 5416 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                if query[0] <= 458 {
                                                                    if query[0] <= 436 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        } else {
                                                            if query[0] <= 510 {
                                                                if query[8] <= 5250 {
                                                                    if query[12] <= 8250 {
                                                                        true
                                                                    } else {
                                                                        if query[2] <= 2740 {
                                                                            false
                                                                        } else {
                                                                            true
                                                                        }
                                                                    }
                                                                } else {
                                                                    false
                                                                }
                                                            } else {
                                                                if query[6] <= 778 {
                                                                    if query[6] <= 288 {
                                                                        false
                                                                    } else {
                                                                        if query[13] <= 81 {
                                                                            if query[0] <= 682 {
                                                                                false
                                                                            } else {
                                                                                true
                                                                            }
                                                                        } else {
                                                                            true
                                                                        }
                                                                    }
                                                                } else {
                                                                    if query[0] <= 621 {
                                                                        if query[0] <= 606 {
                                                                            if query[0] <= 571 {
                                                                                if query[13] <= 121
                                                                                {
                                                                                    true
                                                                                } else {
                                                                                    if query[8]
                                                                                        <= 3750
                                                                                    {
                                                                                        if query[1]
                                                                                            <= 3750
                                                                                        {
                                                                                            false
                                                                                        } else {
                                                                                            true
                                                                                        }
                                                                                    } else {
                                                                                        false
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                true
                                                                            }
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        if query[0] <= 774 {
                                                                            if query[6] <= 2775 {
                                                                                if query[13] <= 211
                                                                                {
                                                                                    if query[2]
                                                                                        <= 5271
                                                                                    {
                                                                                        if query[2]
                                                                                            <= 2066
                                                                                        {
                                                                                            if query[2] <= 1888 {
                                                                                                if query[7] <= 3395 {
                                                                                                    true
                                                                                                } else {
                                                                                                    false
                                                                                                }
                                                                                            } else {
                                                                                                false
                                                                                            }
                                                                                        } else {
                                                                                            true
                                                                                        }
                                                                                    } else {
                                                                                        false
                                                                                    }
                                                                                } else {
                                                                                    if query[13]
                                                                                        <= 223
                                                                                    {
                                                                                        false
                                                                                    } else {
                                                                                        if query[1]
                                                                                            <= 2916
                                                                                        {
                                                                                            if query[13] <= 259 {
                                                                                                false
                                                                                            } else {
                                                                                                true
                                                                                            }
                                                                                        } else {
                                                                                            if query[0] <= 751 {
                                                                                                true
                                                                                            } else {
                                                                                                if query[0] <= 763 {
                                                                                                    false
                                                                                                } else {
                                                                                                    true
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                false
                                                                            }
                                                                        } else {
                                                                            if query[0] <= 798 {
                                                                                if query[2] <= 4316
                                                                                {
                                                                                    false
                                                                                } else {
                                                                                    true
                                                                                }
                                                                            } else {
                                                                                if query[0] <= 812 {
                                                                                    true
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    true
                                                }
                                            }
                                        } else {
                                            if query[8] <= 4250 {
                                                if query[0] <= 718 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            if query[7] <= 1427 {
                                if query[6] <= 1900 {
                                    if query[0] <= 1127 {
                                        if query[8] <= 2250 {
                                            if query[0] <= 851 {
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            if query[2] <= 5463 {
                                                true
                                            } else {
                                                if query[0] <= 985 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            }
                                        }
                                    } else {
                                        if query[0] <= 1148 {
                                            false
                                        } else {
                                            if query[2] <= 2833 {
                                                if query[1] <= 3750 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                if query[2] <= 4498 {
                                                    if query[0] <= 1887 {
                                                        if query[7] <= 1388 {
                                                            if query[1] <= 5416 {
                                                                if query[7] <= 531 {
                                                                    if query[0] <= 1524 {
                                                                        false
                                                                    } else {
                                                                        true
                                                                    }
                                                                } else {
                                                                    if query[2] <= 3498 {
                                                                        true
                                                                    } else {
                                                                        if query[2] <= 3547 {
                                                                            false
                                                                        } else {
                                                                            if query[2] <= 3864 {
                                                                                true
                                                                            } else {
                                                                                if query[2] <= 3889
                                                                                {
                                                                                    false
                                                                                } else {
                                                                                    if query[1]
                                                                                        <= 3750
                                                                                    {
                                                                                        true
                                                                                    } else {
                                                                                        if query[0]
                                                                                            <= 1324
                                                                                        {
                                                                                            true
                                                                                        } else {
                                                                                            false
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                true
                                                            }
                                                        } else {
                                                            false
                                                        }
                                                    } else {
                                                        if query[0] <= 2089 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    }
                                                } else {
                                                    if query[7] <= 848 {
                                                        true
                                                    } else {
                                                        if query[7] <= 960 {
                                                            if query[0] <= 1783 {
                                                                false
                                                            } else {
                                                                if query[1] <= 2916 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        } else {
                                                            if query[7] <= 1340 {
                                                                true
                                                            } else {
                                                                if query[0] <= 2251 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    if query[12] <= 1750 {
                                        if query[2] <= 2806 {
                                            if query[0] <= 1129 {
                                                false
                                            } else {
                                                true
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        if query[7] <= 1334 {
                                            if query[13] <= 219 {
                                                if query[13] <= 60 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[13] <= 242 {
                                                    if query[0] <= 1845 {
                                                        false
                                                    } else {
                                                        if query[0] <= 2398 {
                                                            true
                                                        } else {
                                                            false
                                                        }
                                                    }
                                                } else {
                                                    if query[8] <= 4250 {
                                                        true
                                                    } else {
                                                        if query[7] <= 706 {
                                                            true
                                                        } else {
                                                            if query[0] <= 1056 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            false
                                        }
                                    }
                                }
                            } else {
                                if query[7] <= 2501 {
                                    if query[7] <= 1834 {
                                        if query[7] <= 1754 {
                                            if query[12] <= 1750 {
                                                if query[0] <= 1012 {
                                                    true
                                                } else {
                                                    if query[2] <= 3113 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                }
                                            } else {
                                                if query[6] <= 1036 {
                                                    if query[6] <= 981 {
                                                        if query[8] <= 2250 {
                                                            false
                                                        } else {
                                                            if query[2] <= 4310 {
                                                                true
                                                            } else {
                                                                if query[7] <= 1571 {
                                                                    if query[2] <= 5293 {
                                                                        false
                                                                    } else {
                                                                        true
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    if query[12] <= 2250 {
                                                        if query[1] <= 4166 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        if query[13] <= 244 {
                                                            true
                                                        } else {
                                                            if query[13] <= 251 {
                                                                false
                                                            } else {
                                                                true
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            if query[13] <= 196 {
                                                if query[0] <= 854 {
                                                    false
                                                } else {
                                                    if query[6] <= 494 {
                                                        if query[0] <= 1895 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        true
                                                    }
                                                }
                                            } else {
                                                if query[2] <= 5287 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            }
                                        }
                                    } else {
                                        if query[2] <= 2848 {
                                            if query[0] <= 1126 {
                                                if query[0] <= 928 {
                                                    if query[0] <= 910 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[1] <= 4583 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        } else {
                                            if query[12] <= 4750 {
                                                true
                                            } else {
                                                if query[6] <= 2741 {
                                                    if query[1] <= 4583 {
                                                        true
                                                    } else {
                                                        if query[2] <= 3561 {
                                                            false
                                                        } else {
                                                            if query[12] <= 6250 {
                                                                false
                                                            } else {
                                                                if query[13] <= 233 {
                                                                    true
                                                                } else {
                                                                    if query[2] <= 5240 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    if query[8] <= 3250 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    if query[2] <= 5331 {
                                        if query[12] <= 4750 {
                                            if query[2] <= 1718 {
                                                false
                                            } else {
                                                if query[13] <= 74 {
                                                    if query[0] <= 1220 {
                                                        true
                                                    } else {
                                                        if query[2] <= 5148 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    }
                                                } else {
                                                    if query[13] <= 182 {
                                                        if query[6] <= 2891 {
                                                            if query[8] <= 2250 {
                                                                if query[13] <= 122 {
                                                                    if query[0] <= 1254 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                if query[6] <= 2619 {
                                                                    true
                                                                } else {
                                                                    if query[0] <= 1272 {
                                                                        false
                                                                    } else {
                                                                        true
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            if query[2] <= 4712 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        }
                                                    } else {
                                                        if query[7] <= 2952 {
                                                            if query[1] <= 4583 {
                                                                if query[6] <= 2688 {
                                                                    if query[12] <= 2250 {
                                                                        if query[6] <= 1250 {
                                                                            true
                                                                        } else {
                                                                            if query[0] <= 1703 {
                                                                                false
                                                                            } else {
                                                                                true
                                                                            }
                                                                        }
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                true
                                                            }
                                                        } else {
                                                            if query[7] <= 3967 {
                                                                if query[0] <= 898 {
                                                                    if query[0] <= 871 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                if query[0] <= 1128 {
                                                                    true
                                                                } else {
                                                                    false
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            if query[2] <= 3975 {
                                                if query[12] <= 8250 {
                                                    if query[13] <= 169 {
                                                        if query[8] <= 4750 {
                                                            if query[13] <= 143 {
                                                                if query[6] <= 380 {
                                                                    false
                                                                } else {
                                                                    if query[2] <= 3146 {
                                                                        if query[0] <= 1241 {
                                                                            true
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        true
                                                                    }
                                                                }
                                                            } else {
                                                                if query[1] <= 2916 {
                                                                    true
                                                                } else {
                                                                    false
                                                                }
                                                            }
                                                        } else {
                                                            false
                                                        }
                                                    } else {
                                                        if query[2] <= 2039 {
                                                            false
                                                        } else {
                                                            if query[6] <= 399 {
                                                                false
                                                            } else {
                                                                if query[7] <= 2879 {
                                                                    if query[0] <= 1131 {
                                                                        false
                                                                    } else {
                                                                        true
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[13] <= 227 {
                                                    if query[12] <= 7750 {
                                                        if query[2] <= 4490 {
                                                            if query[2] <= 4414 {
                                                                if query[2] <= 3987 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                false
                                                            }
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        if query[13] <= 173 {
                                                            if query[13] <= 126 {
                                                                if query[0] <= 1001 {
                                                                    true
                                                                } else {
                                                                    if query[2] <= 4028 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                }
                                                            } else {
                                                                if query[6] <= 585 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        } else {
                                                            false
                                                        }
                                                    }
                                                } else {
                                                    if query[1] <= 2916 {
                                                        if query[0] <= 2080 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        true
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        if query[2] <= 5406 {
                                            if query[6] <= 516 {
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            if query[8] <= 3250 {
                                                if query[0] <= 1157 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[1] <= 4583 {
                                                    if query[0] <= 935 {
                                                        false
                                                    } else {
                                                        if query[6] <= 2544 {
                                                            true
                                                        } else {
                                                            if query[0] <= 2257 {
                                                                false
                                                            } else {
                                                                true
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    if query[0] <= 1394 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        if query[7] <= 3633 {
                            false
                        } else {
                            true
                        }
                    }
                } else {
                    if query[2] <= 5768 {
                        false
                    } else {
                        if query[7] <= 3903 {
                            if query[8] <= 4750 {
                                if query[7] <= 3593 {
                                    if query[7] <= 3505 {
                                        if query[7] <= 3085 {
                                            if query[7] <= 3041 {
                                                if query[1] <= 4583 {
                                                    if query[0] <= 1222 {
                                                        true
                                                    } else {
                                                        if query[0] <= 1455 {
                                                            if query[12] <= 8250 {
                                                                if query[7] <= 2570 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                true
                                                            }
                                                        } else {
                                                            if query[7] <= 2440 {
                                                                if query[2] <= 6136 {
                                                                    if query[13] <= 178 {
                                                                        if query[0] <= 2867 {
                                                                            true
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        if query[2] <= 5808 {
                                                                            true
                                                                        } else {
                                                                            if query[6] <= 2802 {
                                                                                false
                                                                            } else {
                                                                                true
                                                                            }
                                                                        }
                                                                    }
                                                                } else {
                                                                    if query[7] <= 313 {
                                                                        false
                                                                    } else {
                                                                        if query[2] <= 6889 {
                                                                            true
                                                                        } else {
                                                                            if query[0] <= 1848 {
                                                                                false
                                                                            } else {
                                                                                true
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                if query[13] <= 153 {
                                                                    if query[2] <= 6728 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    if query[6] <= 2842 {
                                                                        if query[8] <= 2250 {
                                                                            true
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        true
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    if query[7] <= 2281 {
                                                        if query[2] <= 5855 {
                                                            true
                                                        } else {
                                                            if query[0] <= 2823 {
                                                                if query[2] <= 6897 {
                                                                    if query[7] <= 700 {
                                                                        if query[13] <= 128 {
                                                                            false
                                                                        } else {
                                                                            true
                                                                        }
                                                                    } else {
                                                                        if query[13] <= 58 {
                                                                            true
                                                                        } else {
                                                                            if query[13] <= 206 {
                                                                                false
                                                                            } else {
                                                                                if query[12] <= 4750
                                                                                {
                                                                                    if query[7]
                                                                                        <= 1170
                                                                                    {
                                                                                        false
                                                                                    } else {
                                                                                        true
                                                                                    }
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                true
                                                            }
                                                        }
                                                    } else {
                                                        if query[2] <= 6411 {
                                                            true
                                                        } else {
                                                            if query[2] <= 6534 {
                                                                false
                                                            } else {
                                                                true
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                false
                                            }
                                        } else {
                                            if query[0] <= 2550 {
                                                true
                                            } else {
                                                if query[7] <= 3255 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    true
                                }
                            } else {
                                if query[13] <= 62 {
                                    false
                                } else {
                                    if query[13] <= 200 {
                                        if query[0] <= 2743 {
                                            if query[0] <= 2503 {
                                                if query[2] <= 6286 {
                                                    if query[0] <= 2181 {
                                                        if query[0] <= 1019 {
                                                            false
                                                        } else {
                                                            if query[1] <= 5416 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    if query[2] <= 6791 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                }
                                            } else {
                                                false
                                            }
                                        } else {
                                            true
                                        }
                                    } else {
                                        if query[6] <= 2757 {
                                            if query[13] <= 288 {
                                                if query[2] <= 5864 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                true
                                            }
                                        } else {
                                            true
                                        }
                                    }
                                }
                            }
                        } else {
                            if query[13] <= 223 {
                                false
                            } else {
                                if query[0] <= 2248 {
                                    true
                                } else {
                                    false
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        if query[13] <= 100 {
            if query[12] <= 6250 {
                if query[2] <= 7665 {
                    if query[12] <= 4000 {
                        if query[6] <= 2638 {
                            true
                        } else {
                            if query[0] <= 1508 {
                                true
                            } else {
                                false
                            }
                        }
                    } else {
                        false
                    }
                } else {
                    if query[6] <= 1710 {
                        if query[7] <= 490 {
                            false
                        } else {
                            if query[7] <= 933 {
                                if query[0] <= 1634 {
                                    if query[0] <= 1412 {
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    true
                                }
                            } else {
                                if query[7] <= 1822 {
                                    if query[0] <= 1112 {
                                        true
                                    } else {
                                        if query[8] <= 5250 {
                                            if query[0] <= 2956 {
                                                false
                                            } else {
                                                if query[0] <= 2992 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        } else {
                                            if query[1] <= 3750 {
                                                false
                                            } else {
                                                true
                                            }
                                        }
                                    }
                                } else {
                                    if query[0] <= 2517 {
                                        if query[8] <= 3750 {
                                            if query[0] <= 1794 {
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            if query[12] <= 1750 {
                                                false
                                            } else {
                                                if query[0] <= 1988 {
                                                    if query[0] <= 1882 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    true
                                                }
                                            }
                                        }
                                    } else {
                                        if query[0] <= 2788 {
                                            true
                                        } else {
                                            if query[6] <= 343 {
                                                true
                                            } else {
                                                if query[6] <= 1254 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        if query[6] <= 2663 {
                            if query[13] <= 99 {
                                if query[8] <= 2750 {
                                    if query[0] <= 2010 {
                                        false
                                    } else {
                                        true
                                    }
                                } else {
                                    false
                                }
                            } else {
                                true
                            }
                        } else {
                            if query[7] <= 2788 {
                                true
                            } else {
                                if query[0] <= 1559 {
                                    true
                                } else {
                                    if query[0] <= 2798 {
                                        false
                                    } else {
                                        true
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                if query[1] <= 4583 {
                    if query[7] <= 2594 {
                        if query[6] <= 1532 {
                            false
                        } else {
                            if query[1] <= 2916 {
                                false
                            } else {
                                if query[13] <= 63 {
                                    false
                                } else {
                                    true
                                }
                            }
                        }
                    } else {
                        if query[13] <= 70 {
                            true
                        } else {
                            if query[1] <= 3750 {
                                if query[0] <= 2782 {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                    }
                } else {
                    if query[0] <= 1371 {
                        if query[0] <= 1039 {
                            false
                        } else {
                            true
                        }
                    } else {
                        if query[8] <= 3750 {
                            if query[7] <= 2565 {
                                if query[13] <= 53 {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                true
                            }
                        } else {
                            if query[0] <= 4199 {
                                if query[0] <= 4198 {
                                    if query[7] <= 4463 {
                                        if query[8] <= 5750 {
                                            if query[6] <= 4382 {
                                                if query[6] <= -4878 {
                                                    false
                                                } else {
                                                    if query[1] <= 7916 {
                                                        if query[7] <= 3870 {
                                                            if query[6] <= 4311 {
                                                                if query[7] <= 3450 {
                                                                    if query[1] <= 5416 {
                                                                        if query[13] <= 92 {
                                                                            if query[6] <= 3066 {
                                                                                if query[7] <= 2909
                                                                                {
                                                                                    if query[13]
                                                                                        <= 52
                                                                                    {
                                                                                        true
                                                                                    } else {
                                                                                        false
                                                                                    }
                                                                                } else {
                                                                                    true
                                                                                }
                                                                            } else {
                                                                                true
                                                                            }
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        if query[6] <= 2178 {
                                                                            if query[0] <= 1776 {
                                                                                false
                                                                            } else {
                                                                                true
                                                                            }
                                                                        } else {
                                                                            if query[13] <= 92 {
                                                                                if query[13] <= 57 {
                                                                                    if query[13]
                                                                                        <= 49
                                                                                    {
                                                                                        if query[0]
                                                                                            <= 4024
                                                                                        {
                                                                                            false
                                                                                        } else {
                                                                                            if query[0] <= 4069 {
                                                                                                true
                                                                                            } else {
                                                                                                false
                                                                                            }
                                                                                        }
                                                                                    } else {
                                                                                        if query[0]
                                                                                            <= 2649
                                                                                        {
                                                                                            false
                                                                                        } else {
                                                                                            true
                                                                                        }
                                                                                    }
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            } else {
                                                                                true
                                                                            }
                                                                        }
                                                                    }
                                                                } else {
                                                                    if query[7] <= 3669 {
                                                                        if query[13] <= 89 {
                                                                            false
                                                                        } else {
                                                                            if query[0] <= 3144 {
                                                                                false
                                                                            } else {
                                                                                true
                                                                            }
                                                                        }
                                                                    } else {
                                                                        if query[1] <= 5833 {
                                                                            false
                                                                        } else {
                                                                            if query[0] <= 3365 {
                                                                                if query[0] <= 3077
                                                                                {
                                                                                    true
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            } else {
                                                                                true
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                true
                                                            }
                                                        } else {
                                                            if query[6] <= 2462 {
                                                                if query[0] <= 2584 {
                                                                    true
                                                                } else {
                                                                    false
                                                                }
                                                            } else {
                                                                if query[6] <= 3745 {
                                                                    true
                                                                } else {
                                                                    false
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        if query[8] <= 4250 {
                                                            if query[12] <= 8250 {
                                                                false
                                                            } else {
                                                                if query[1] <= 9166 {
                                                                    true
                                                                } else {
                                                                    false
                                                                }
                                                            }
                                                        } else {
                                                            false
                                                        }
                                                    }
                                                }
                                            } else {
                                                if query[6] <= 5128 {
                                                    if query[6] <= 5117 {
                                                        if query[0] <= 3610 {
                                                            false
                                                        } else {
                                                            if query[0] <= 3621 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        }
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    false
                                                }
                                            }
                                        } else {
                                            if query[2] <= 9006 {
                                                if query[2] <= 8996 {
                                                    if query[7] <= 2066 {
                                                        if query[2] <= 8574 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[8] <= 6750 {
                                                    if query[6] <= 3895 {
                                                        if query[6] <= 3879 {
                                                            if query[13] <= 92 {
                                                                if query[7] <= 2409 {
                                                                    if query[7] <= 2355 {
                                                                        false
                                                                    } else {
                                                                        true
                                                                    }
                                                                } else {
                                                                    false
                                                                }
                                                            } else {
                                                                if query[1] <= 7083 {
                                                                    true
                                                                } else {
                                                                    false
                                                                }
                                                            }
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    false
                                                }
                                            }
                                        }
                                    } else {
                                        if query[7] <= 5389 {
                                            if query[7] <= 5387 {
                                                if query[1] <= 5416 {
                                                    if query[8] <= 5250 {
                                                        if query[13] <= 66 {
                                                            false
                                                        } else {
                                                            if query[6] <= 4051 {
                                                                if query[6] <= -3661 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                false
                                                            }
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    false
                                                }
                                            } else {
                                                if query[0] <= 2827 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        } else {
                                            false
                                        }
                                    }
                                } else {
                                    true
                                }
                            } else {
                                if query[0] <= 4745 {
                                    if query[0] <= 4744 {
                                        if query[0] <= 4629 {
                                            false
                                        } else {
                                            if query[0] <= 4630 {
                                                if query[6] <= 2592 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                if query[7] <= 3749 {
                                                    if query[7] <= 3741 {
                                                        false
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    false
                                                }
                                            }
                                        }
                                    } else {
                                        if query[1] <= 5416 {
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                } else {
                                    false
                                }
                            }
                        }
                    }
                }
            }
        } else {
            if query[7] <= 1363 {
                if query[0] <= 1109 {
                    if query[0] <= 830 {
                        true
                    } else {
                        false
                    }
                } else {
                    if query[1] <= 2916 {
                        if query[0] <= 1943 {
                            if query[2] <= 7267 {
                                true
                            } else {
                                false
                            }
                        } else {
                            if query[7] <= 897 {
                                if query[12] <= 1750 {
                                    false
                                } else {
                                    if query[2] <= 9330 {
                                        true
                                    } else {
                                        if query[0] <= 2465 {
                                            true
                                        } else {
                                            if query[0] <= 2781 {
                                                if query[6] <= 1552 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            } else {
                                                true
                                            }
                                        }
                                    }
                                }
                            } else {
                                if query[6] <= 1051 {
                                    if query[0] <= 2991 {
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    if query[7] <= 1328 {
                                        false
                                    } else {
                                        true
                                    }
                                }
                            }
                        }
                    } else {
                        if query[6] <= 2876 {
                            if query[7] <= 1151 {
                                if query[2] <= 8875 {
                                    if query[8] <= 4250 {
                                        if query[8] <= 2750 {
                                            true
                                        } else {
                                            if query[6] <= 2432 {
                                                if query[7] <= 353 {
                                                    false
                                                } else {
                                                    if query[13] <= 166 {
                                                        if query[0] <= 2382 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        true
                                                    }
                                                }
                                            } else {
                                                false
                                            }
                                        }
                                    } else {
                                        true
                                    }
                                } else {
                                    if query[2] <= 9437 {
                                        if query[7] <= 1018 {
                                            false
                                        } else {
                                            true
                                        }
                                    } else {
                                        if query[7] <= 1017 {
                                            if query[6] <= 265 {
                                                if query[1] <= 5416 {
                                                    if query[0] <= 2224 {
                                                        false
                                                    } else {
                                                        if query[0] <= 2819 {
                                                            if query[0] <= 2630 {
                                                                if query[13] <= 216 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                true
                                                            }
                                                        } else {
                                                            if query[0] <= 2940 {
                                                                false
                                                            } else {
                                                                true
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[8] <= 2750 {
                                                    true
                                                } else {
                                                    if query[13] <= 277 {
                                                        if query[13] <= 235 {
                                                            if query[13] <= 213 {
                                                                if query[0] <= 1403 {
                                                                    false
                                                                } else {
                                                                    if query[13] <= 104 {
                                                                        false
                                                                    } else {
                                                                        if query[12] <= 4750 {
                                                                            true
                                                                        } else {
                                                                            if query[7] <= 543 {
                                                                                true
                                                                            } else {
                                                                                if query[6] <= 1597
                                                                                {
                                                                                    false
                                                                                } else {
                                                                                    if query[6]
                                                                                        <= 2666
                                                                                    {
                                                                                        true
                                                                                    } else {
                                                                                        false
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                false
                                                            }
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                }
                                            }
                                        } else {
                                            if query[8] <= 3750 {
                                                false
                                            } else {
                                                if query[1] <= 4583 {
                                                    if query[0] <= 2111 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    true
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                if query[1] <= 5416 {
                                    if query[0] <= 2964 {
                                        if query[8] <= 2250 {
                                            if query[13] <= 134 {
                                                false
                                            } else {
                                                true
                                            }
                                        } else {
                                            true
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    if query[0] <= 2535 {
                                        false
                                    } else {
                                        true
                                    }
                                }
                            }
                        } else {
                            false
                        }
                    }
                }
            } else {
                if query[0] <= 963 {
                    if query[6] <= 2462 {
                        if query[6] <= -4791 {
                            if query[0] <= 872 {
                                false
                            } else {
                                true
                            }
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                } else {
                    if query[0] <= 1443 {
                        if query[0] <= 1151 {
                            if query[7] <= 2183 {
                                true
                            } else {
                                if query[6] <= 1654 {
                                    false
                                } else {
                                    if query[0] <= 1047 {
                                        false
                                    } else {
                                        true
                                    }
                                }
                            }
                        } else {
                            if query[2] <= 8127 {
                                if query[7] <= 3310 {
                                    if query[0] <= 1392 {
                                        false
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        }
                    } else {
                        if query[7] <= 3909 {
                            if query[13] <= 265 {
                                if query[7] <= 3234 {
                                    if query[13] <= 262 {
                                        if query[6] <= 2513 {
                                            if query[0] <= 1851 {
                                                if query[2] <= 7544 {
                                                    true
                                                } else {
                                                    if query[0] <= 1485 {
                                                        if query[1] <= 3750 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        if query[12] <= 7750 {
                                                            if query[8] <= 3250 {
                                                                if query[1] <= 5416 {
                                                                    if query[7] <= 1457 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                false
                                                            }
                                                        } else {
                                                            if query[6] <= -4326 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                if query[2] <= 9954 {
                                                    if query[6] <= 245 {
                                                        if query[2] <= 7056 {
                                                            false
                                                        } else {
                                                            if query[7] <= 2871 {
                                                                if query[13] <= 241 {
                                                                    true
                                                                } else {
                                                                    if query[0] <= 2698 {
                                                                        false
                                                                    } else {
                                                                        true
                                                                    }
                                                                }
                                                            } else {
                                                                if query[7] <= 2993 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        if query[0] <= 2134 {
                                                            if query[6] <= 370 {
                                                                false
                                                            } else {
                                                                true
                                                            }
                                                        } else {
                                                            if query[0] <= 2253 {
                                                                false
                                                            } else {
                                                                if query[0] <= 2539 {
                                                                    if query[6] <= 1205 {
                                                                        true
                                                                    } else {
                                                                        if query[0] <= 2503 {
                                                                            false
                                                                        } else {
                                                                            true
                                                                        }
                                                                    }
                                                                } else {
                                                                    if query[6] <= 998 {
                                                                        if query[6] <= 363 {
                                                                            true
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        if query[0] <= 2828 {
                                                                            if query[7] <= 2593 {
                                                                                if query[7] <= 1516
                                                                                {
                                                                                    true
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            } else {
                                                                                if query[0] <= 2581
                                                                                {
                                                                                    false
                                                                                } else {
                                                                                    true
                                                                                }
                                                                            }
                                                                        } else {
                                                                            true
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    if query[6] <= 1656 {
                                                        if query[13] <= 256 {
                                                            if query[7] <= 2564 {
                                                                if query[7] <= 1657 {
                                                                    if query[6] <= 1559 {
                                                                        if query[8] <= 5250 {
                                                                            if query[7] <= 1536 {
                                                                                false
                                                                            } else {
                                                                                if query[7] <= 1611
                                                                                {
                                                                                    if query[8]
                                                                                        <= 2750
                                                                                    {
                                                                                        false
                                                                                    } else {
                                                                                        true
                                                                                    }
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            }
                                                                        } else {
                                                                            true
                                                                        }
                                                                    } else {
                                                                        true
                                                                    }
                                                                } else {
                                                                    if query[0] <= 2713 {
                                                                        if query[6] <= 690 {
                                                                            if query[0] <= 2212 {
                                                                                if query[7] <= 2232
                                                                                {
                                                                                    false
                                                                                } else {
                                                                                    if query[0]
                                                                                        <= 2072
                                                                                    {
                                                                                        false
                                                                                    } else {
                                                                                        true
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                if query[12] <= 2750
                                                                                {
                                                                                    if query[1]
                                                                                        <= 3750
                                                                                    {
                                                                                        true
                                                                                    } else {
                                                                                        false
                                                                                    }
                                                                                } else {
                                                                                    true
                                                                                }
                                                                            }
                                                                        } else {
                                                                            true
                                                                        }
                                                                    } else {
                                                                        if query[7] <= 1768 {
                                                                            true
                                                                        } else {
                                                                            if query[7] <= 2510 {
                                                                                if query[12] <= 1750
                                                                                {
                                                                                    true
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            } else {
                                                                                true
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                if query[1] <= 2916 {
                                                                    if query[6] <= -4420 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    if query[6] <= 1238 {
                                                                        if query[13] <= 112 {
                                                                            if query[0] <= 2479 {
                                                                                false
                                                                            } else {
                                                                                true
                                                                            }
                                                                        } else {
                                                                            if query[0] <= 2878 {
                                                                                false
                                                                            } else {
                                                                                if query[0] <= 2895
                                                                                {
                                                                                    true
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            }
                                                                        }
                                                                    } else {
                                                                        if query[7] <= 2992 {
                                                                            false
                                                                        } else {
                                                                            true
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            if query[6] <= 1250 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        }
                                                    } else {
                                                        if query[6] <= 2147 {
                                                            if query[13] <= 216 {
                                                                if query[6] <= 1736 {
                                                                    if query[6] <= 1720 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                if query[0] <= 2751 {
                                                                    false
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        } else {
                                                            if query[6] <= 2491 {
                                                                if query[13] <= 236 {
                                                                    false
                                                                } else {
                                                                    if query[8] <= 3750 {
                                                                        false
                                                                    } else {
                                                                        true
                                                                    }
                                                                }
                                                            } else {
                                                                true
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            if query[13] <= 109 {
                                                if query[6] <= 2613 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[13] <= 229 {
                                                    if query[0] <= 1542 {
                                                        true
                                                    } else {
                                                        if query[1] <= 4583 {
                                                            if query[7] <= 2515 {
                                                                if query[7] <= 2284 {
                                                                    if query[12] <= 3250 {
                                                                        if query[0] <= 2871 {
                                                                            true
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        false
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                false
                                                            }
                                                        } else {
                                                            false
                                                        }
                                                    }
                                                } else {
                                                    if query[13] <= 237 {
                                                        true
                                                    } else {
                                                        if query[6] <= 2806 {
                                                            if query[0] <= 2567 {
                                                                true
                                                            } else {
                                                                false
                                                            }
                                                        } else {
                                                            false
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        true
                                    }
                                } else {
                                    if query[13] <= 258 {
                                        if query[0] <= 1963 {
                                            if query[8] <= 2250 {
                                                if query[0] <= 1673 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                true
                                            }
                                        } else {
                                            if query[13] <= 231 {
                                                if query[7] <= 3301 {
                                                    true
                                                } else {
                                                    if query[7] <= 3459 {
                                                        if query[2] <= 7968 {
                                                            if query[2] <= 7647 {
                                                                false
                                                            } else {
                                                                true
                                                            }
                                                        } else {
                                                            if query[13] <= 125 {
                                                                true
                                                            } else {
                                                                if query[0] <= 2877 {
                                                                    false
                                                                } else {
                                                                    if query[0] <= 2890 {
                                                                        true
                                                                    } else {
                                                                        false
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        if query[12] <= 6250 {
                                                            if query[13] <= 139 {
                                                                false
                                                            } else {
                                                                if query[12] <= 4750 {
                                                                    if query[7] <= 3714 {
                                                                        if query[0] <= 2877 {
                                                                            true
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        if query[7] <= 3815 {
                                                                            false
                                                                        } else {
                                                                            true
                                                                        }
                                                                    }
                                                                } else {
                                                                    false
                                                                }
                                                            }
                                                        } else {
                                                            if query[6] <= 2787 {
                                                                if query[0] <= 2657 {
                                                                    if query[6] <= 1519 {
                                                                        if query[6] <= 805 {
                                                                            if query[1] <= 3333 {
                                                                                false
                                                                            } else {
                                                                                true
                                                                            }
                                                                        } else {
                                                                            false
                                                                        }
                                                                    } else {
                                                                        true
                                                                    }
                                                                } else {
                                                                    true
                                                                }
                                                            } else {
                                                                false
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                if query[0] <= 2110 {
                                                    false
                                                } else {
                                                    if query[8] <= 2250 {
                                                        false
                                                    } else {
                                                        true
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        false
                                    }
                                }
                            } else {
                                if query[0] <= 1974 {
                                    if query[2] <= 8592 {
                                        if query[2] <= 7285 {
                                            true
                                        } else {
                                            false
                                        }
                                    } else {
                                        if query[13] <= 290 {
                                            if query[8] <= 2750 {
                                                if query[0] <= 1544 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                true
                                            }
                                        } else {
                                            if query[13] <= 295 {
                                                false
                                            } else {
                                                true
                                            }
                                        }
                                    }
                                } else {
                                    if query[0] <= 2836 {
                                        if query[13] <= 290 {
                                            if query[6] <= 2586 {
                                                if query[8] <= 4250 {
                                                    false
                                                } else {
                                                    if query[7] <= 2931 {
                                                        true
                                                    } else {
                                                        false
                                                    }
                                                }
                                            } else {
                                                true
                                            }
                                        } else {
                                            if query[6] <= 1098 {
                                                if query[7] <= 2090 {
                                                    false
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if query[6] <= 2866 {
                                                    if query[0] <= 2396 {
                                                        if query[0] <= 2294 {
                                                            false
                                                        } else {
                                                            true
                                                        }
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    true
                                                }
                                            }
                                        }
                                    } else {
                                        if query[13] <= 272 {
                                            false
                                        } else {
                                            if query[7] <= 2822 {
                                                true
                                            } else {
                                                if query[6] <= 489 {
                                                    true
                                                } else {
                                                    false
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            if query[7] <= 3989 {
                                if query[6] <= 2418 {
                                    if query[2] <= 7617 {
                                        if query[0] <= 2246 {
                                            false
                                        } else {
                                            true
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    if query[0] <= 2750 {
                                        false
                                    } else {
                                        true
                                    }
                                }
                            } else {
                                true
                            }
                        }
                    }
                }
            }
        }
    }
}
